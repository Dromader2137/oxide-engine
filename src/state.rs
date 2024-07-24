use crate::{
    input::InputManager, rendering::{Renderer, Window}, vulkan::{context::VulkanContext, memory::MemoryAllocators}
};

pub struct State {
    pub window: Window,
    pub input: InputManager,
    pub vulkan_context: VulkanContext,
    pub memory_allocators: MemoryAllocators,
    pub renderer: Renderer,
    pub time: f64,
    pub delta_time: f64,
    pub physics_time_scale: f32
}
