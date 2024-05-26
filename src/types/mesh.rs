use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::VertexData, state::State};

pub struct Mesh {
    pub vertices: Vec<VertexData>,
    pub vertex_buffer: Subbuffer<[VertexData]>
}

impl Mesh {
    pub fn new(vertices: Vec<VertexData>, state: &State) -> Mesh {
        if vertices.len() == 0 {
            panic!("empty slice not allowed");
        }

        Mesh {
            vertices: vertices.clone(),
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
