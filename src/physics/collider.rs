use hecs::Entity;

use crate::types::transform::Transform;

#[derive(Debug, Clone)]
pub enum Collider {
    Sphere(f64)
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

pub fn sphere_to_sphere(a: Entity, a_r: f64, a_t: &Transform, b: Entity, b_r: f64, b_t: &Transform) -> Option<Collision> {
    let dst = (a_t.position - b_t.position).length() - a_r - b_r;
    if dst <= 0.0 {
        return Some(Collision {
            entity_a: a,
            entity_b: b
        });
    }
    None
}
