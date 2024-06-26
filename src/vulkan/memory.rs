use std::sync::Arc;

use vulkano::{command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, descriptor_set::allocator::StandardDescriptorSetAllocator, memory::allocator::StandardMemoryAllocator};

use super::context::VulkanContext;

pub struct MemoryAllocators {
    pub standard_memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl MemoryAllocators {
    pub fn new(context: &VulkanContext) -> MemoryAllocators {
        let standard_memory_allocator = Arc::new(StandardMemoryAllocator::new_default(context.device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            context.device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            context.device.clone(),
            Default::default(),
        ));

        MemoryAllocators {
            standard_memory_allocator,
            command_buffer_allocator, 
            descriptor_set_allocator
        }
    }
}
