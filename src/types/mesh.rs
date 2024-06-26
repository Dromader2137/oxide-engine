use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulkano::{pipeline::{Pipeline, PipelineBindPoint}, buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};
use log::{debug, error};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, loaders::{gltf::load_gltf, obj::load_obj}, rendering::{rendering_component::RenderingComponent, VertexData}, state::State};

use super::material::Attachment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    #[serde(skip)]
    pub vertex_buffer: Option<Subbuffer<[VertexData]>>,
    #[serde(skip)]
    pub index_buffer: Option<Subbuffer<[u32]>>
}

impl Mesh {
    pub fn new(name: &str, vertices: Vec<VertexData>, indices: Vec<u32>) -> Mesh {
        if vertices.len() == 0 {
            panic!("Empty vertex list not allowed!");
        }
        if indices.len() == 0 {
            panic!("Empty index list not allowed!");
        }
        if *indices.iter().max().unwrap() as usize >= vertices.len() {
            panic!("Index larger than vertex buffer length!");
        }

        Mesh {
            name: name.to_string(),
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: None,
            index_buffer: None
        }
    }

    pub fn load(&mut self, state: &State) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
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
                self.vertices.clone()
            ).unwrap()
        );
        self.index_buffer = Some(
            Buffer::from_iter(
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
                self.indices.clone()
            ).unwrap()
        )
    }
}

pub struct DynamicMesh {
    pub material: Uuid,
    pub mesh: Option<Uuid>,
}

impl DynamicMesh {
    pub fn new(material: Uuid) -> Self {
        Self { mesh: None, material }
    }
}

pub struct DynamicMeshRenderingComponent {}

impl RenderingComponent for DynamicMeshRenderingComponent {
    fn render(&self,
            mut builder:
                AutoCommandBufferBuilder<
                    PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                    StandardCommandBufferAllocator
                >,
            _world: &World,
            assets: &AssetLibrary,
            state: &State,
            image_id: usize
        ) -> AutoCommandBufferBuilder<
                PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                StandardCommandBufferAllocator
            > 
    {
        for (key, entry) in state.renderer.dynamic_mesh_data.iter() {
            if entry.vertex_ptr.is_none() || entry.model.is_none() || entry.indirect_draw.is_none() {
                continue;
            }

            let material = assets.materials.get(key).unwrap();
            let pipeline = state
                .renderer
                .pipelines
                .get(&(
                        material.vertex_shader,
                        material.fragment_shader
                ))
                .unwrap()
                .clone();

            builder.bind_pipeline_graphics(pipeline.clone()).unwrap();

            let vp_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().first().unwrap().clone(),
                [WriteDescriptorSet::buffer(
                    0,
                    state.renderer.vp_buffers.get(image_id).unwrap().clone(),
                )],
                [],
            )
                .unwrap();

            let m_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().get(1).unwrap().clone(),
                [WriteDescriptorSet::buffer(
                    0,
                    entry.model.as_ref().unwrap().clone(),
                )],
                [],
            )
                .unwrap();

            let vertex_set = PersistentDescriptorSet::new(
                state.memory_allocators.descriptor_set_allocator.as_ref(),
                pipeline.layout().set_layouts().get(2).unwrap().clone(),
                [
                WriteDescriptorSet::buffer(0, entry.vertex_ptr.as_ref().unwrap().clone()),
                WriteDescriptorSet::buffer(1, entry.index_ptr.as_ref().unwrap().clone()),
                ],
                [],
            )
                .unwrap();

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
                                let (_, tex) = assets.textures.iter().find(|(_, x)| x.name == "default".to_string()).unwrap();
                                WriteDescriptorSet::image_view_sampler(
                                    id as u32,
                                    tex.image_view.as_ref().unwrap().clone(),
                                    tex.sampler.as_ref().unwrap().clone(),
                                )
                            }
                        })
                        .collect::<Vec<_>>(),
                        [],
                    )
                        .unwrap()
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
                        [
                        WriteDescriptorSet::buffer(
                            0,
                            material.parameter_buffer.as_ref().unwrap().clone(),
                        )
                        ],
                        [],
                    ).unwrap()
                )
            } else {
                None
            };


            let mut sets = vec![vp_set, m_set, vertex_set];
            if attachment_set.is_some() {
                sets.push(attachment_set.unwrap());
            }
            if material_set.is_some() {
                sets.push(material_set.unwrap())
            }

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    sets
                )
                .unwrap();

            builder
                .draw_indirect(entry.indirect_draw.as_ref().unwrap().clone())
                .unwrap();
            };

        builder
    }
    
}

pub struct MeshBufferLoader {}

impl System for MeshBufferLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for (_, mesh) in assets.meshes.iter_mut() {
            mesh.load(state);
        }
    }

    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
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


