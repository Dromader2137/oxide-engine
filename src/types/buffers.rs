use std::sync::Arc;

use bytemuck::Pod;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::sync::{now, GpuFuture};

use crate::rendering::Renderer;
use crate::state::State;

#[derive(Clone)]
pub struct UpdatableBuffer<DataType> {
    pub buffer: Subbuffer<DataType>,
    staging_buffer: Subbuffer<DataType>,
    command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>
}

impl<DataType> UpdatableBuffer<DataType>
where
    DataType: Pod + BufferContents,
{
    pub fn new(renderer: &Renderer, buffer_usage: BufferUsage) -> UpdatableBuffer<DataType> {
        let mut updatable_buffer = UpdatableBuffer::<DataType> { 
            buffer:
                Buffer::new_sized(
                    renderer.memeory_allocator.as_ref().unwrap().clone(), 
                    BufferCreateInfo {
                        usage: buffer_usage | BufferUsage::TRANSFER_DST,
                        ..Default::default()
                    }, 
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                        ..Default::default()
                    }
                ).unwrap(),
            staging_buffer: 
                Buffer::new_sized(
                    renderer.memeory_allocator.as_ref().unwrap().clone(), 
                    BufferCreateInfo {
                        usage: buffer_usage | BufferUsage::TRANSFER_SRC,
                        ..Default::default()
                    }, 
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                        ..Default::default()
                    }
                ).unwrap(),
            command_buffer: None
        };
        updatable_buffer.command_buffer = Some({
            let command_buffer_allocator = StandardCommandBufferAllocator::new(
                renderer.device.as_ref().unwrap().clone(),
                Default::default(),
            );

            let mut builder = AutoCommandBufferBuilder::primary(
                &command_buffer_allocator,
                renderer.queue.as_ref().unwrap().queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            ).unwrap();
            builder
                .copy_buffer(
                    CopyBufferInfo::buffers(updatable_buffer.staging_buffer.clone(), updatable_buffer.buffer.clone())
                )
                .unwrap();

            builder.build().unwrap()
        });
        updatable_buffer
    }

    pub fn write(&self, state: &State, data: DataType) {
        let mut content = self.staging_buffer.write().unwrap();
        *content = data;
        drop(content);

        let future = now(state.renderer.device.as_ref().unwrap().clone())
            .then_execute(state.renderer.queue.as_ref().unwrap().clone(), self.command_buffer.as_ref().unwrap().clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        // future.wait(None).unwrap();

    }
    
    pub fn write_all(&self, _state: &State, data: DataType) {
        self.write(_state, data);
    }
}
