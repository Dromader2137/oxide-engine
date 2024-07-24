use vulkano::command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};

use crate::{asset_library::AssetLibrary, ecs::World, state::State};

pub trait RenderingComponent {
    fn render(
        &self,
        builder:
            AutoCommandBufferBuilder<
                PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                StandardCommandBufferAllocator
            >,
        _world: &World,
        _assets: &AssetLibrary,
        _state: &State,
        _image_id: usize
        ) -> AutoCommandBufferBuilder<
                PrimaryAutoCommandBuffer<StandardCommandBufferAllocator>, 
                StandardCommandBufferAllocator
            > {
        builder
    }
}
