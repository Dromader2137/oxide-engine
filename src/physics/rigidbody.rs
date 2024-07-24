use crate::{ecs::System, types::{transform::Transform, vectors::Vec3f}};

#[derive(Debug, Clone)]
pub struct Rigidbody {
    pub mass: f32,
    pub velocity: Vec3f
}

pub struct RigidbodyHandler {}

impl System for RigidbodyHandler {
    fn on_start(&self, _world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, _state: &mut crate::state::State) {}

    fn on_update(&self, world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, state: &mut crate::state::State) {
        for (_, (rigidbody, transform)) in world.entities.query::<(&Rigidbody, &mut Transform)>().iter() {
            let delta_pos = rigidbody.velocity.to_vec3d() * state.delta_time * state.physics_time_scale as f64;
            transform.position += delta_pos;
        }
    }
}
