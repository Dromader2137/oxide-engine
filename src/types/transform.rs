use vulkano::buffer::BufferUsage;

use crate::{rendering::Renderer, types::vectors::*};

use super::{buffers::UpdatableBuffer, matrices::Matrix4f};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3d,
    pub scale: Vec3f,
    pub rotation: Vec3f,
    pub buffer: Option<UpdatableBuffer<Matrix4f>>,
}

impl Transform {
    pub fn new(pos: Vec3d, scl: Vec3f, rot: Vec3f) -> Transform {
        Transform {
            position: pos,
            scale: scl,
            rotation: rot,
            buffer: None,
        }
    }

    pub fn load(&mut self, renderer: &Renderer) {
        self.buffer = Some(UpdatableBuffer::new(renderer, BufferUsage::UNIFORM_BUFFER));
    }
}
