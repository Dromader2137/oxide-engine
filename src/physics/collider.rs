use log::debug;

use crate::types::transform::Transform;

#[derive(Debug, Clone)]
pub enum Collider {
    Sphere(f64)
}

#[derive(Debug, Clone)]
pub struct Collision {}

pub fn sphere_to_sphere(a_r: f64, a_t: &Transform, b_r: f64, b_t: &Transform) {
    let dst = (a_t.position - b_t.position).length() - a_r - b_r;
    if dst <= 0.0 {
        debug!("Collision");
    }
}
