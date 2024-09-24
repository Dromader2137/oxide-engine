use crate::{asset_library::AssetLibrary, ecs::{System, World}, state::State};

use super::{matrices::Matrix4f, transform::Transform, vectors::Vec3f};

#[derive(Clone, Copy)]
pub struct Camera {
    pub vfov: f32,
    pub near: f32,
}

pub struct CameraUpdater {}

impl System for CameraUpdater {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        let entities = world.entities.borrow_mut();

        let mut query = entities.query::<(&Camera, &Transform)>();
        let transform_data = query.iter().next().expect("Camera with trasform not found!").1.1;
        let cam_rot = transform_data.rotation;
        state.renderer.vp_pos = transform_data.position;
        state.renderer.vp_data.view = Matrix4f::look_at(
            Vec3f::new([0.0, 0.0, 0.0]),
            cam_rot * Vec3f::new([0.0, 0.0, -1.0]),
            cam_rot * Vec3f::new([0.0, 1.0, 0.0]),
        );
    }
}
