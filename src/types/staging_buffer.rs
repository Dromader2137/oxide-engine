use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::state::State;

pub struct StagingBuffer<T> {
    main: Subbuffer<T>,
    staging: Subbuffer<T>
}

impl<T> StagingBuffer<T> 
    where T: BufferContents
{
    fn new(data: T, size: u64, usage: BufferUsage, memory: MemoryTypeFilter, state: &State) -> StagingBuffer<T> {
        StagingBuffer::<T> {
            main: Buffer::new_unsized::<T>(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: usage | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: memory,
                    ..Default::default()
                },
                size
            ).unwrap(),
            staging: Buffer::from_data(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: memory,
                    ..Default::default()
                },
                data
            ).unwrap(),
        }

    }

}
