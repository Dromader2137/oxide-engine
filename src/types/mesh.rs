use std::{sync::{mpsc::{self, Receiver, Sender}, Arc}, thread::{self}};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, device::Device, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};
use log::{debug, error};

use crate::{assets::asset_library::AssetLibrary, ecs::{System, World}, loaders::{gltf::load_gltf, obj::load_obj}, rendering::VertexData, state::State};

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
                        usage: BufferUsage::VERTEX_BUFFER,
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
                        usage: BufferUsage::INDEX_BUFFER,
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
                usage: BufferUsage::VERTEX_BUFFER,
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
                usage: BufferUsage::INDEX_BUFFER,
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

