use crate::{asset_library::AssetLibrary, ecs::{System, World}, state::State};

use super::{matrices::Matrix4f, transform::Transform, vectors::Vec3f};

#[derive(Clone, Copy)]
pub struct Camera {
    pub vfov: f32,
    pub near: f32,
    pub far: f32,
}

pub struct CameraUpdater {}

impl System for CameraUpdater {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        let mut camera = world.borrow_component_vec_mut::<Camera>().unwrap();
        let mut transform = world.borrow_component_vec_mut::<Transform>().unwrap();
        let zip = camera.iter_mut().zip(transform.iter_mut());
        let mut iter =
            zip.filter_map(|(camera, transform)| Some((camera.as_mut()?, transform.as_mut()?)));
        let (_, transform_data) = iter.next().unwrap();
        let cam_rot = Matrix4f::rotation_xzy(transform_data.rotation);
        state.renderer.vp_data.view = Matrix4f::look_at(
            transform_data.position.to_vec3f(),
            cam_rot.vec_mul(Vec3f::new([1.0, 0.0, 0.0])),
            cam_rot.vec_mul(Vec3f::new([0.0, 1.0, 0.0])),
        );
        state
            .renderer
            .vp_buffer
            .as_mut()
            .unwrap()
            .write(state.renderer.vp_data);
    }
}
