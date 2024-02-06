use vulkano::buffer::BufferUsage;

use crate::{types::vectors::*, rendering::Renderer};

use super::{buffers::UpdatableBuffer, matrices::Matrix4f};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3d,
    pub buffer: Option<UpdatableBuffer<Matrix4f>>
}

impl Transform {
    pub fn new(pos: Vec3d) -> Transform {
        Transform { 
            position: pos,
            buffer: None
        }
    }

    pub fn load(&mut self, renderer: &Renderer) {
        self.buffer = Some(UpdatableBuffer::new(renderer.device.as_ref().unwrap(), BufferUsage::UNIFORM_BUFFER));
    }
}
