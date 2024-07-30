use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, sync::{mpsc::{self, Receiver, Sender}, Arc}, thread::{self}};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, DrawIndirectCommand, PrimaryAutoCommandBuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, device::Device, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{Pipeline, PipelineBindPoint}};
use log::{debug, error};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, loaders::{gltf::load_gltf, obj::load_obj}, rendering::{rendering_component::RenderingComponent, PipelineIdentifier, VertexData}, state::State};

use super::{material::Attachment, matrices::Matrix4f, model::ModelComponent, transform::{ModelData, Transform}};

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    #[serde(skip)]
    pub vertex_buffer: Option<Arc<Subbuffer<[VertexData]>>>,
    #[serde(skip)]
    pub index_buffer: Option<Arc<Subbuffer<[u32]>>>,
    #[serde(skip)]
    pub transfer_requested: bool,
    #[serde(skip)]
    pub transfering: bool,
    #[serde(skip)]
    pub new_vertices: Option<Vec<VertexData>>,
    #[serde(skip)]
    pub new_indices: Option<Vec<u32>>,
}

#[derive(Debug)]
struct MeshUpdateSubmitData {
    uuid: Uuid,
    vertices: Vec<VertexData>,
    indices: Vec<u32>,
}

type MeshSubbuffers = (Uuid, Subbuffer<[VertexData]>, Subbuffer<[u32]>, Vec<VertexData>, Vec<u32>);

fn run_worker(
    device: Arc<Device>,
    work_recv: Receiver<MeshUpdateSubmitData>,
    buffer_send: Sender<MeshSubbuffers>
) {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    thread::spawn(move || {
        while let Ok(submit_data) = work_recv.recv() {
            let res = (
                submit_data.uuid,
                Buffer::from_iter(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    submit_data.vertices.clone()
                ).unwrap(),
                Buffer::from_iter(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    submit_data.indices.clone()
                ).unwrap(),
                submit_data.vertices,
                submit_data.indices
            );
            
            let _ = memory_allocator.clone();

            buffer_send.send(res).unwrap();
        }
    });
}

impl Mesh {
    pub fn new(name: &str, vertices: Vec<VertexData>, indices: Vec<u32>) -> Mesh {
        if vertices.is_empty() {
            panic!("Empty vertex list not allowed!");
        }
        if indices.is_empty() {
            panic!("Empty index list not allowed!");
        }
        if *indices.iter().max().unwrap() as usize >= vertices.len() {
            error!("{} {}", indices.iter().max().unwrap(), vertices.len());
            panic!("Index larger than vertex buffer length!");
        }

        Mesh {
            name: name.to_string(),
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: None,
            index_buffer: None,
            transfer_requested: false,
            transfering: false,
            new_vertices: None,
            new_indices: None
        }
    }

    pub fn load(&mut self, _state: &State, vertices: Vec<VertexData>, indices: Vec<u32>) {
        self.transfer_requested = true;
        self.new_vertices = Some(vertices);
        self.new_indices = Some(indices);
    }

    pub fn load_immidiate(&mut self, state: &State, vertices: Vec<VertexData>, indices: Vec<u32>) {
        self.vertices.clone_from(&vertices);
        self.indices.clone_from(&indices);
        self.vertex_buffer = Some(Arc::new(Buffer::from_iter(
            state.memory_allocators.standard_memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices
        ).unwrap()));
        self.index_buffer = Some(Arc::new(Buffer::from_iter(
            state.memory_allocators.standard_memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices
        ).unwrap()));
    }
}

pub struct DynamicMesh {
    pub material: Uuid,
    pub material_name: String,
    pub mesh: Option<Uuid>,
}

impl DynamicMesh {
    pub fn new(material_name: String) -> Self {
        Self { mesh: None, material_name, material: Uuid::nil() }
    }

    pub fn load_material(&mut self, assets: &AssetLibrary) {
        self.material = *assets.materials.iter().find(|(_, v)| v.name == self.material_name).expect("Material not found").0;
    }
}

fn prepare_meshes(world: &World, assets: &AssetLibrary, state: &State, material: Uuid, dynamic_mesh_data: &mut HashMap<Uuid, MeshBuffers>) {
    let entities = world.entities.borrow_mut();

    let mut dynamic_query = entities.query::<(&mut DynamicMesh, &Transform)>();
    let mut filtered_by_material: Vec<_> = dynamic_query.iter().filter(|x| x.1.0.material == material).collect();

    let pmb = match dynamic_mesh_data.get_mut(&material) {
        Some(val) => val,
        None => {
            dynamic_mesh_data.insert(material, MeshBuffers::new());
            dynamic_mesh_data.get_mut(&material).unwrap()
        }
    };

    pmb.index.clear();
    pmb.vertex.clear();

    let camera_pos = state.renderer.vp_pos;
    filtered_by_material.sort_by(|a, b| {
        (a.1 .1.position - camera_pos)
            .length_sqr()
            .total_cmp(&(b.1 .1.position - camera_pos).length_sqr())
    });

    let mut counter      = 0_u32;
    let mut vertex_ptr   = Vec::new();
    let mut index_ptr    = Vec::new();
    let mut model        = Vec::new();
    let mut indirect     = Vec::new();

    for (_, (mesh, transform)) in filtered_by_material {
        if mesh.mesh.is_none() {
            continue;
        }

        let mesh_data = assets.meshes.get(&mesh.mesh.unwrap()).expect("Mesh not found");
        if mesh_data.vertex_buffer.is_none() {continue;}
        if mesh_data.index_buffer.is_none() {continue;}

        vertex_ptr.push(mesh_data.vertex_buffer.as_ref().unwrap().device_address().unwrap().get());
        index_ptr.push(mesh_data.index_buffer.as_ref().unwrap().device_address().unwrap().get());
        pmb.vertex.push(mesh_data.vertex_buffer.as_ref().unwrap().clone());
        pmb.index.push(mesh_data.index_buffer.as_ref().unwrap().clone());

        model.push(ModelData {
            translation: Matrix4f::translation((transform.position - camera_pos).to_vec3f()),
            rotation: transform.rotation.to_matrix(),
            scale: Matrix4f::scale(transform.scale)
        });
        indirect.push(DrawIndirectCommand {
            instance_count: 1,
            first_instance: counter,
            vertex_count: mesh_data.indices.len() as u32,
            first_vertex: 0,
        });

        counter += 1;
    }

    for (_, (model_comp, transform)) in entities.query::<(&ModelComponent, &Transform)>().iter() {
        for (mesh_name, _) in assets.models.get(&model_comp.model_uuid).unwrap().meshes_and_materials.iter().filter(|x| x.1 == material) {
            let mesh_data = assets.meshes.get(mesh_name).expect("Mesh not found");
            if mesh_data.vertex_buffer.is_none() {continue;}
            if mesh_data.index_buffer.is_none() {continue;}
        
            vertex_ptr.push(mesh_data.vertex_buffer.as_ref().unwrap().device_address().unwrap().get());
            index_ptr.push(mesh_data.index_buffer.as_ref().unwrap().device_address().unwrap().get());
            pmb.vertex.push(mesh_data.vertex_buffer.as_ref().unwrap().clone());
            pmb.index.push(mesh_data.index_buffer.as_ref().unwrap().clone());

            model.push(ModelData {
                translation: Matrix4f::translation((transform.position - camera_pos).to_vec3f()),
                rotation: transform.rotation.to_matrix(),
                scale: Matrix4f::scale(transform.scale)
            });
            indirect.push(DrawIndirectCommand {
                instance_count: 1,
                first_instance: counter,
                vertex_count: mesh_data.indices.len() as u32,
                first_vertex: 0,
            });

            counter += 1;
        }
    }

    pmb.model = if !model.is_empty() {
        Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                model,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.vertex_ptr = if !vertex_ptr.is_empty() {
        Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertex_ptr,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.index_ptr = if !index_ptr.is_empty() {
        Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                index_ptr,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.indirect_draw = if !indirect.is_empty() {
        Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDIRECT_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indirect,
            )
            .unwrap(),
        )
    } else {
        None
    };
}

#[derive(Clone)]
pub struct MeshBuffers {
    vertex: Vec<Arc<Subbuffer<[VertexData]>>>,
    index: Vec<Arc<Subbuffer<[u32]>>>,
    pub vertex_ptr: Option<Subbuffer<[u64]>>,
    pub index_ptr: Option<Subbuffer<[u64]>>,
    pub model: Option<Subbuffer<[ModelData]>>,
    pub indirect_draw: Option<Subbuffer<[DrawIndirectCommand]>>,
}

impl MeshBuffers {
    pub fn new() -> MeshBuffers {
        MeshBuffers {
            vertex: Vec::new(),
            index: Vec::new(),
            vertex_ptr: None,
            index_ptr: None,
            indirect_draw: None,
            model: None,
        }
    }
}

impl Default for MeshBuffers {
    fn default() -> Self {
        Self::new()
    }
}


pub struct DynamicMeshRenderingComponent {
    pub dynamic_mesh_data: RefCell<HashMap<Uuid, MeshBuffers>>,
}

impl RenderingComponent for DynamicMeshRenderingComponent {
    fn render(
            &self,
            mut builder:
                AutoCommandBufferBuilder<
                    PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                    StandardCommandBufferAllocator
                >,
            world: &World,
            assets: &AssetLibrary,
            state: &State,
            image_id: usize
        ) -> AutoCommandBufferBuilder<
                PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                StandardCommandBufferAllocator
            > 
    {
        for (material_uuid, _) in assets.materials.iter() {
            prepare_meshes(world, assets, state, *material_uuid, self.dynamic_mesh_data.borrow_mut().borrow_mut());
        }

        for (key, entry) in self.dynamic_mesh_data.borrow().iter() {
            let material = assets.materials.get(key).unwrap();

            if entry.vertex_ptr.is_none() || entry.index_ptr.is_none() || entry.model.is_none() || entry.indirect_draw.is_none() {
                continue;
            }

            let pipeline = state.renderer.pipelines.get(&PipelineIdentifier::new(material.vertex_shader, material.fragment_shader, material.rendering_type)).unwrap().clone();

            builder.bind_pipeline_graphics(pipeline.clone()).unwrap();

            let vp_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().first().unwrap().clone(),
                [WriteDescriptorSet::buffer(0, state.renderer.vp_buffers.get(image_id).unwrap().clone())],
                [],
            ).unwrap();

            let m_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().get(1).unwrap().clone(),
                [WriteDescriptorSet::buffer(0, entry.model.as_ref().unwrap().clone(),)],
                [],
            ).unwrap();

            let vertex_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().get(2).unwrap().clone(),
                [
                    WriteDescriptorSet::buffer(0, entry.vertex_ptr.as_ref().unwrap().clone()),
                    WriteDescriptorSet::buffer(1, entry.index_ptr.as_ref().unwrap().clone()),
                ],
                [],
            ).unwrap();

            let attachment_set = if !material.attachments.is_empty() {
                Some({
                    PersistentDescriptorSet::new(
                        state.memory_allocators.descriptor_set_allocator.as_ref(),
                        pipeline.layout().set_layouts().get(3).unwrap().clone(),
                        material
                        .attachments
                        .iter()
                        .enumerate()
                        .map(|(id, attachment)| match attachment {
                            Attachment::Texture(uuid) => {
                                let tex = assets.textures.get(uuid).unwrap();
                                WriteDescriptorSet::image_view_sampler(
                                    id as u32,
                                    tex.image_view.as_ref().unwrap().clone(),
                                    tex.sampler.as_ref().unwrap().clone(),
                                )
                            },
                            Attachment::DefaultTexture => {
                                let (_, tex) = assets.textures.iter().find(|(_, x)| x.name == *"default").unwrap();
                                WriteDescriptorSet::image_view_sampler(
                                    id as u32,
                                    tex.image_view.as_ref().unwrap().clone(),
                                    tex.sampler.as_ref().unwrap().clone(),
                                )
                            }
                        }).collect::<Vec<_>>(),
                        [],
                    ).unwrap()
                })
            } else {
                None
            };

            let material_set = if material.parameter_buffer.is_some() {
                Some(
                    PersistentDescriptorSet::new(
                        state.memory_allocators.descriptor_set_allocator.as_ref(),
                        pipeline.layout().set_layouts().get({
                            if attachment_set.is_some() {
                                4
                            } else {
                                3
                            }
                        }).unwrap().clone(),
                        [WriteDescriptorSet::buffer(0, material.parameter_buffer.as_ref().unwrap().clone())],
                        [],
                    ).unwrap()
                )
            } else {
                None
            };


            let mut sets = vec![vp_set, m_set, vertex_set];
            if let Some(attachment_set) = attachment_set {
                sets.push(attachment_set);
            }
            if let Some(material_set) = material_set {
                sets.push(material_set);
            }

            builder
                .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, sets).unwrap();

            builder
                .draw_indirect(entry.indirect_draw.as_ref().unwrap().clone())
                .unwrap();
        };

        builder
    }
    
}

pub struct MeshBufferLoader {
    work_send: Sender<MeshUpdateSubmitData>,
    buffer_recv: Receiver<MeshSubbuffers>
}

impl MeshBufferLoader {
    pub fn new(state: &mut State) -> MeshBufferLoader {
        let (work_send, work_recv) = mpsc::channel();
        let (buffer_send, buffer_recv) = mpsc::channel();
        run_worker(state.vulkan_context.device.clone(), work_recv, buffer_send);
        MeshBufferLoader {
            work_send,
            buffer_recv
        }
    }
}

impl System for MeshBufferLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for (_, mesh) in assets.meshes.iter_mut() {
            mesh.load_immidiate(state, mesh.vertices.clone(), mesh.indices.clone());
        }
    }

    fn on_update(&self, _world: &World, assets: &mut AssetLibrary, _state: &mut State) {
        while let Ok(ret_data) = self.buffer_recv.try_recv() {
            let (uuid, vertex, index, vertices, indices) = ret_data;
            if let Some(mesh) = assets.meshes.get_mut(&uuid) {
                mesh.vertex_buffer = Some(Arc::new(vertex));
                mesh.index_buffer = Some(Arc::new(index));
                mesh.vertices = vertices;
                mesh.indices = indices;
                mesh.transfering = false;
            }
        }

        for (uuid, mesh) in assets.meshes.iter_mut() {
            if mesh.transfer_requested && !mesh.transfering {
                mesh.transfer_requested = false;
                mesh.transfering = true;
                let data = MeshUpdateSubmitData {
                    uuid: *uuid,
                    vertices: mesh.new_vertices.take().unwrap(),
                    indices: mesh.new_indices.take().unwrap()
                };
                self.work_send.send(data).unwrap();
            }
        }
    }
}

pub fn load_model_meshes(assets: &mut AssetLibrary) {
    let len = assets.models.len();
    for i in 0..len {
        let model_name = assets.models.values().nth(i).unwrap().name.clone();
        debug!("Loading model {}", model_name);
        let mam = match model_name.split_once('.') {
            Some((name, "obj")) => load_obj(name.to_string(), assets).expect("Failed to load"),
            Some((name, "gltf")) => load_gltf(name.to_string(), assets).expect("Failed to load"),
            Some(_) => {
                error!("Unsupportes format {}", model_name);
                continue;
            },
            None => {
                error!("Invalid format");
                continue;
            }
        };
        assets.models.values_mut().nth(i).unwrap().meshes_and_materials = mam;
    }
}

pub struct DynamicMeshMaterialLoader {}

impl System for DynamicMeshMaterialLoader {
    fn on_start(&self, world: &World, assets: &mut AssetLibrary, _state: &mut State) {
        let entities = world.entities.borrow_mut();

        for (_, mesh) in entities.query::<&mut DynamicMesh>().iter() {
            mesh.load_material(assets);
        }
    }

    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}

