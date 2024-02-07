use bytemuck::Pod;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

use crate::rendering::Renderer;

#[derive(Clone, Debug)]
pub struct UpdatableBuffer<DataType> {
    pub main_buffer: Subbuffer<DataType>,
    pub staging_buffer: Subbuffer<DataType>,
}

impl<DataType> UpdatableBuffer<DataType>
where
    DataType: Pod + BufferContents,
{
    pub fn new(renderer: &Renderer, buffer_usage: BufferUsage) -> UpdatableBuffer<DataType> {
        UpdatableBuffer {
            main_buffer: Buffer::new_sized(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: buffer_usage | BufferUsage::TRANSFER_DST,
                    // size: std::mem::size_of::<DataType>() as u64,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            )
            .unwrap(),
            staging_buffer: Buffer::new_sized(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: buffer_usage | BufferUsage::TRANSFER_SRC,
                    // size: std::mem::size_of::<DataType>() as u64,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                },
            )
            .unwrap(),
        }
    }

    pub fn write(&mut self, data: DataType) {
        match self.staging_buffer.write() {
            Ok(mut content) => {
                *content = data;
            }
            Err(_) => println!("Failed buffer write!"),
        }
    }
}
