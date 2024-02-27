use bytemuck::{Zeroable, Pod};
use vulkano::buffer::BufferUsage;

use crate::{rendering::{Renderer, Window, ShaderManager}, types::vectors::*, ecs::{System, World}};

use super::{buffers::UpdatableBuffer, matrices::Matrix4f};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3d,
    pub scale: Vec3f,
    pub rotation: Vec3f,
    pub buffer: Option<UpdatableBuffer<ModelData>>,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct ModelData {
    model: Matrix4f,
    rotation: Matrix4f
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
        self.update_buffer();
    }

    pub fn update_buffer(&mut self) {
        self.buffer
            .as_mut()
            .unwrap()
            .write(ModelData{
                model: Matrix4f::translation(self.position.to_vec3f()) *
                    Matrix4f::rotation_yxz(self.rotation) *
                    Matrix4f::scale(self.scale),
                rotation:
                    Matrix4f::rotation_yxz(self.rotation)
                }
            );
    }
}

pub struct TransformUpdater {}

impl System for TransformUpdater {
    fn on_start(&self, world: &World, renderer: &mut Renderer, _window: &Window, _shaders: &ShaderManager) {
        for transform in world
            .borrow_component_vec_mut::<Transform>()
            .unwrap()
            .iter_mut()
            .filter(|x| x.is_some())
        {
                transform.as_mut().unwrap().load(renderer);
        }
    }

    fn on_update(&self, world: &World, _renderer: &mut Renderer, _window: &Window, _shaders: &ShaderManager) { 
        for transform in world
            .borrow_component_vec_mut::<Transform>()
            .unwrap()
            .iter_mut() 
            .filter(|x| x.is_some())
        {
                transform.as_mut().unwrap().update_buffer();
        }
    }
}
