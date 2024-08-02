use crate::{ecs::System, types::{transform::Transform, vectors::Vec3f}};

#[derive(Debug, Clone)]
pub struct Rigidbody {
    pub mass: f32,
    pub velocity: Vec3f,
    pub angular_velocity: Vec3f,
}

impl Rigidbody {
    pub fn new(m: f32, v: Vec3f, w: Vec3f) -> Rigidbody {
        Rigidbody { 
            mass: m, 
            velocity: v, 
            angular_velocity: w, 
        }
    }
}

pub struct RigidbodyHandler {}

impl System for RigidbodyHandler {
    fn on_start(&self, _world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, _state: &mut crate::state::State) {}

    fn on_update(&self, world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, state: &mut crate::state::State) {
        let entities = world.entities.borrow_mut();

        for (_, (rigidbody, transform)) in entities.query::<(&Rigidbody, &mut Transform)>().iter() {
            let delta_pos = rigidbody.velocity.to_vec3d() * state.delta_time * state.physics_time_scale as f64;
            transform.position += delta_pos;
        }
    }
}
