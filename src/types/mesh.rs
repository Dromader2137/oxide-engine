use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::VertexData, state::State};

pub struct Mesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Subbuffer<[VertexData]>,
    pub index_buffer: Subbuffer<[u32]>
}

impl Mesh {
    pub fn new(vertices: Vec<VertexData>, indices: Vec<u32>, state: &State) -> Mesh {
        if vertices.len() == 0 {
            panic!("Empty vertex list not allowed!");
        }
        if indices.len() == 0 {
            panic!("Empty index list not allowed!");
        }

        Mesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
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
            ).unwrap(),
            index_buffer: Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
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
            ).unwrap()
        }
    }
}

pub struct DynamicMesh {
    pub material: String,
    pub mesh: Option<String>,
}

impl DynamicMesh {
    pub fn new(material: String) -> Self {
        Self { mesh: None, material }
    }
}

pub struct DynamicMeshLoader {}

impl System for DynamicMeshLoader {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
