use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::{now, GpuFuture}};

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
                    usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
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

#[derive(Debug, Clone)]
pub struct DynamicMesh {
    pub vertices: Vec<VertexData>,
    pub material: String,
    pub buffer_id: Option<u32>,
    pub changed: bool
}

impl DynamicMesh {
    pub fn from_mesh(mesh: String, assets: &AssetLibrary) -> DynamicMesh {
        let mesh = assets.meshes.iter().find(|x| x.name == mesh).unwrap();
        DynamicMesh {
            vertices: mesh.vertices.clone(),
            material: mesh.material.clone(),
            buffer_id: None,
            changed: true
        }
    }
}

pub struct DynamicMeshLoader {}

impl System for DynamicMeshLoader {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
