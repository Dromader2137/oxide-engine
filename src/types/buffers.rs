use bytemuck::Pod;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

use crate::rendering::Renderer;
use crate::state::State;

#[derive(Clone)]
pub struct UpdatableBuffer<DataType> {
    pub buffer: Subbuffer<DataType>,
}

impl<DataType> UpdatableBuffer<DataType>
where
    DataType: Pod + BufferContents,
{
    pub fn new(renderer: &Renderer, buffer_usage: BufferUsage) -> UpdatableBuffer<DataType> {
        let updatable_buffer = UpdatableBuffer::<DataType> { 
            buffer:
                Buffer::new_sized(
                    renderer.memeory_allocator.clone(), 
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
        };
        updatable_buffer
    }

    pub fn write(&self, _state: &State, data: DataType) {
        let mut content = self.buffer.write().unwrap();
        *content = data;
    }
    
    pub fn write_all(&self, _state: &State, data: DataType) {
        self.write(_state, data);
    }
}
