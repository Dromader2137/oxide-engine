use bytemuck::{Pod, Zeroable};

use crate::{
    assets::asset_library::AssetLibrary,
    ecs::{System, World},
    state::State,
    types::{quaternion::Quat, vectors::*},
};

use super::{matrices::Matrix4f, position::Position};

#[derive(Clone)]
pub struct Transform {
    pub position: Position,
    pub scale: Vec3f,
    pub rotation: Quat,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct ModelData {
    pub translation: Matrix4f,
    pub rotation: Matrix4f,
    pub scale: Matrix4f,
}

impl Transform {
    pub fn new(position: Position, scale: Vec3f, rotation: Quat) -> Transform {
        Transform {
            position,
            scale,
            rotation,
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
