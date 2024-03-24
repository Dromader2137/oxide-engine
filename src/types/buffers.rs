use bytemuck::Pod;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

use crate::rendering::Renderer;
use crate::state::State;

#[derive(Clone, Debug)]
pub struct UpdatableBuffer<DataType> {
    pub buffers: Vec<Subbuffer<DataType>>,
}

impl<DataType> UpdatableBuffer<DataType>
where
    DataType: Pod + BufferContents,
{
    pub fn new(renderer: &Renderer, buffer_usage: BufferUsage) -> UpdatableBuffer<DataType> {
        let mut updatable_buffer = UpdatableBuffer::<DataType> { buffers: Vec::new() };
        for _ in 0..renderer.frames_in_flight {
            updatable_buffer.buffers.push(
                Buffer::new_sized(
                    renderer.memeory_allocator.as_ref().unwrap().clone(), 
                    BufferCreateInfo {
                        usage: buffer_usage,
                        ..Default::default()
                    }, 
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                        ..Default::default()
                    }
                ).unwrap()
            );
        }
        updatable_buffer
    }

    pub fn write(&self, state: &State, data: DataType) {
        match self.buffers.get(state.renderer.previous_fence).unwrap().write() {
            Ok(mut content) => {
                *content = data;
            }
            Err(_) => println!("Failed buffer write!"),
        }
    }
    
    pub fn write_all(&self, _state: &State, data: DataType) {
        for buffer in self.buffers.iter() {
            match buffer.write() {
                Ok(mut content) => {
                    *content = data;
                }
                Err(_) => println!("Failed buffer write!"),
            }
        }
    }
}
