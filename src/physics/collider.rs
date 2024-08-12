use hecs::Entity;

use crate::types::{transform::Transform, vectors::Vec3d};

use super::rigidbody::Rigidbody;

#[derive(Debug, Clone)]
pub enum Collider {
    Sphere(f64)
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub move_a: Vec3d,
    pub move_b: Vec3d,
}

pub fn sphere_to_sphere(
    a: (Entity, &Transform, &Rigidbody, f64), 
    b: (Entity, &Transform, &Rigidbody, f64)
) -> Option<Collision> {
    let dst_centers = (a.1.position - b.1.position).length();
    let dst = dst_centers - a.3 - b.3;

    if dst <= 0.0 {
        let total_mass = a.2.mass + b.2.mass;
        let move_a_norm = (a.1.position - b.1.position) / dst_centers;
        let move_b_norm = (b.1.position - a.1.position) / dst_centers;
        let move_a = move_a_norm * (b.2.mass / total_mass) as f64 * -dst;
        let move_b = move_b_norm * (a.2.mass / total_mass) as f64 * -dst;

        return Some(Collision {
            entity_a: a.0,
            entity_b: b.0,
            move_a,
            move_b
        });
    }
    None
}
