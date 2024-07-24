use bytemuck::{Pod, Zeroable};

use crate::{
    asset_library::AssetLibrary,
    ecs::{System, World},
    state::State,
    types::{quaternion::Quat, vectors::*},
};

use super::matrices::Matrix4f;

#[derive(Clone)]
pub struct Transform {
    pub position: Vec3d,
    pub scale: Vec3f,
    pub rotation: Quat,
    pub changed: bool
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct ModelData {
    pub translation: Matrix4f,
    pub rotation: Matrix4f,
    pub scale: Matrix4f,
}

impl Transform {
    pub fn new(pos: Vec3d, scl: Vec3f, rot: Quat) -> Transform {
        Transform {
            position: pos,
            scale: scl,
            rotation: rot,
            changed: false
        }
    }

    pub fn front(&self) -> Vec3f {
        let f = self.rotation.to_matrix().vec_mul(Vec3f::new([1.0, 0.0, 0.0]));
        Vec3f::new([f.x, f.y, f.z])
    }
    
    pub fn up(&self) -> Vec3f {
        let f = self.rotation.to_matrix().vec_mul(Vec3f::new([0.0, 1.0, 0.0]));
        Vec3f::new([f.x, f.y, f.z])
    }
}

pub struct TransformUpdater {}
impl System for TransformUpdater {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
