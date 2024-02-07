use vulkano::buffer::BufferUsage;

use crate::{rendering::Renderer, types::vectors::*};

use super::{buffers::UpdatableBuffer, matrices::Matrix4f};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3d,
    pub buffer: Option<UpdatableBuffer<Matrix4f>>,
}

impl Transform {
    pub fn new(pos: Vec3d) -> Transform {
        Transform {
            position: pos,
            buffer: None,
        }
    }

    pub fn load(&mut self, renderer: &Renderer) {
        self.buffer = Some(UpdatableBuffer::new(renderer, BufferUsage::UNIFORM_BUFFER));
    }
}
