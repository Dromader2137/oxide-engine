use serde::{Deserialize, Serialize};
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::state::State;

use super::ui_layout::UiVertexData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiMesh {
    pub vertices: Vec<UiVertexData>,
    pub indices: Vec<u32>,
    #[serde(skip)]
    pub vertex_buffer: Option<Subbuffer<[UiVertexData]>>,
    #[serde(skip)]
    pub index_buffer: Option<Subbuffer<[u32]>>
}

impl UiMesh {
    pub fn new(vertices: Vec<UiVertexData>, indices: Vec<u32>) -> UiMesh {
        if vertices.len() == 0 {
            panic!("Empty vertex list not allowed!");
        }
        if indices.len() == 0 {
            panic!("Empty index list not allowed!");
        }
        if *indices.iter().max().unwrap() as usize >= vertices.len() {
            panic!("Index larger than vertex buffer length!");
        }

        UiMesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: None,
            index_buffer: None
        }
    }

    pub fn load(&mut self, state: &State) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.vertices.clone()
            ).unwrap()
        );
        self.index_buffer = Some(
            Buffer::from_iter(
                state.memory_allocators.standard_memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.indices.clone()
            ).unwrap()
        )
    }
}
