use std::sync::Arc;

use bytemuck::Pod;
use vulkano::memory::allocator::{StandardMemoryAllocator, AllocationCreateInfo, MemoryTypeFilter, GenericMemoryAllocatorCreateInfo};
use vulkano::device::Device;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};

#[derive(Clone, Debug)]
pub struct UpdatableBuffer<DataType> {
    pub main_buffer: Subbuffer<DataType>,
    pub staging_buffer: Subbuffer<DataType>
}

impl <DataType> UpdatableBuffer<DataType>
where
    DataType: Pod + BufferContents
{
     pub fn new(device: &Arc<Device>, buffer_usage: BufferUsage) -> UpdatableBuffer<DataType> {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new(
            device.clone(),
            GenericMemoryAllocatorCreateInfo {
                block_sizes: &[256, 256, 256, 256, 256],
                ..Default::default()
            },
        ));

        UpdatableBuffer {
            main_buffer: Buffer::new_sized(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: buffer_usage | BufferUsage::TRANSFER_DST,
                    // size: std::mem::size_of::<DataType>() as u64,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                }
            ).unwrap(),
            staging_buffer: Buffer::new_sized(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: buffer_usage | BufferUsage::TRANSFER_SRC,
                    // size: std::mem::size_of::<DataType>() as u64,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST 
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                }
            ).unwrap()
        }
    }

    pub fn write(&mut self, data: DataType) {
        match self.staging_buffer.write() {
            Ok(mut content) => {
                *content = data;
            },
            Err(_) => println!("Failed buffer write!")
        }
    }
}
