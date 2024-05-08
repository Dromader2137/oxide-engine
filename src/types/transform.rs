use bytemuck::{Pod, Zeroable};
use vulkano::buffer::BufferUsage;

use crate::{
    asset_library::AssetLibrary,
    ecs::{System, World},
    state::State,
    types::vectors::*,
};

use super::{buffers::UpdatableBuffer, matrices::Matrix4f};

#[derive(Clone)]
pub struct Transform {
    pub position: Vec3d,
    pub scale: Vec3f,
    pub rotation: Vec3f,
    pub buffer: Option<UpdatableBuffer<ModelData>>,
    pub changed: bool
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct ModelData {
    pub model: Matrix4f,
    pub rotation: Matrix4f,
}

impl Transform {
    pub fn new(pos: Vec3d, scl: Vec3f, rot: Vec3f) -> Transform {
        Transform {
            position: pos,
            scale: scl,
            rotation: rot,
            buffer: None,
            changed: false
        }
    }

    pub fn load(&mut self, state: &State) {
        self.buffer = Some(UpdatableBuffer::new(&state.renderer, BufferUsage::UNIFORM_BUFFER));
        self.update_buffer(state);
    }

    pub fn update_buffer(&mut self, state: &State) {
        self.buffer.as_mut().unwrap().write_all(state, ModelData {
            model: Matrix4f::translation(self.position.to_vec3f())
                * Matrix4f::rotation_yxz(self.rotation)
                * Matrix4f::scale(self.scale),
            rotation: Matrix4f::rotation_yxz(self.rotation),
        });
    }
}

pub struct TransformUpdater {}

impl System for TransformUpdater {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        for (_, transform) in world.entities.query::<&mut Transform>().iter() {
            transform.load(state);
        }
    }

    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        for (_, transform) in world.entities.query::<&mut Transform>().iter() {
            if transform.changed {
                transform.update_buffer(state);
            }
        }
    }
}
