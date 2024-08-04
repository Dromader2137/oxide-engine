use crate::{ecs::System, types::{quaternion::Quat, transform::Transform, vectors::Vec3f}};

#[derive(Debug, Clone)]
pub struct Rigidbody {
    pub mass: f32,
    pub velocity: Vec3f,
    pub angular_velocity: Vec3f,
    torque: Vec3f,
    force: Vec3f
}

impl Rigidbody {
    pub fn add_torque(&mut self, torque: Vec3f) {
        self.torque += torque;
    }
    
    pub fn add_force(&mut self, force: Vec3f) {
        self.force += force;
    }

    pub fn add_force_at_point(&mut self, force: Vec3f, point: Vec3f) {
        self.torque += point.cross(force);
        self.force += force;
    }
}

impl Rigidbody {
    pub fn new(m: f32, v: Vec3f, w: Vec3f) -> Rigidbody {
        Rigidbody { 
            mass: m, 
            velocity: v, 
            angular_velocity: w, 
            torque: Vec3f::new([0.0, 0.0, 0.0]),
            force: Vec3f::new([0.0, 0.0, 0.0])
        }
    }
}

pub struct RigidbodyHandler {}

impl System for RigidbodyHandler {
    fn on_start(&self, _world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, _state: &mut crate::state::State) {}

    fn on_update(&self, world: &crate::ecs::World, _assets: &mut crate::asset_library::AssetLibrary, state: &mut crate::state::State) {
        let entities = world.entities.borrow_mut();

        for (_, (rigidbody, transform)) in entities.query::<(&mut Rigidbody, &mut Transform)>().iter() {
            rigidbody.velocity += rigidbody.force * state.delta_time as f32 / rigidbody.mass;
            rigidbody.force = Vec3f::new([0.0, 0.0, 0.0]);

            let delta_pos = rigidbody.velocity.to_vec3d() * state.delta_time * state.physics_time_scale as f64;
            transform.position += delta_pos;

            rigidbody.angular_velocity += rigidbody.torque * state.delta_time as f32 / rigidbody.mass;
            rigidbody.torque = Vec3f::new([0.0, 0.0, 0.0]);

            let angular_velocity_quat = Quat::new([0.0, rigidbody.angular_velocity.x, rigidbody.angular_velocity.y, rigidbody.angular_velocity.z]);
            let avtr = angular_velocity_quat * transform.rotation;
            let d_rotation = avtr * (state.delta_time / 2.0) as f32;
            transform.rotation = (transform.rotation + d_rotation).normalize();
        }
    }
}
