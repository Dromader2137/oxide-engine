use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::{now, GpuFuture}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{Renderer, VertexData}, state::State};

pub struct DynamicMesh {
    pub vertices: Vec<VertexData>,
    pub material: String,
    pub buffer_id: Option<u32>,
    pub changed: bool
}

pub struct DynamicMeshLoader {}

impl System for DynamicMeshLoader {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
