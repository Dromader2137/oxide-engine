use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{Renderer, VertexData}, state::State};

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub material: String,
    pub vertex_buffer: Option<Subbuffer<[VertexData]>>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
}

impl Mesh {
    pub fn load(&mut self, renderer: &mut Renderer) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.vertices.clone(),
            )
            .unwrap(),
        );
        self.index_buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.indices.clone(),
            )
            .unwrap(),
        );
    }
}

pub struct MeshLoader {}

impl System for MeshLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for mesh in assets.meshes.iter_mut() {
            mesh.load(&mut state.renderer);
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
