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
    pub changed: bool
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct ModelData {
    pub model: Matrix4f,
    pub rotation: Matrix4f,
    pub position: Vec3f
}

impl Transform {
    pub fn new(pos: Vec3d, scl: Vec3f, rot: Vec3f) -> Transform {
        Transform {
            position: pos,
            scale: scl,
            rotation: rot,
            changed: false
        }
    }
}

pub struct TransformUpdater {}

impl System for TransformUpdater {
    fn on_start(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {}

    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {}
}
