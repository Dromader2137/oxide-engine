use log::debug;

use crate::{ecs::System, types::transform::Transform};

use super::{collider::{sphere_to_sphere, Collider}, rigidbody::Rigidbody};

pub struct CollisionHandler {}

impl System for CollisionHandler {
    fn on_start(
        &self,
        _world: &crate::ecs::World,
        _assets: &mut crate::asset_library::AssetLibrary,
        _state: &mut crate::state::State,
    ) {
    }

    fn on_update(
        &self,
        world: &crate::ecs::World,
        _assets: &mut crate::asset_library::AssetLibrary,
        _state: &mut crate::state::State,
    ) {
        let entities = world.entities.borrow_mut();
        let mut collisions = Vec::new();
        
        {
            let mut query = entities.query::<(&Transform, &Rigidbody, &Collider)>();
            let vec = query.iter().map(|(e, x)| (e, x)).collect::<Vec<_>>();

            for (a, (ta, ra, ca)) in vec.iter() {
                for (b, (tb, rb, cb)) in vec.iter() {
                    if a <= b { continue; }

                    match (ca, cb) {
                        (Collider::Sphere(a_r), Collider::Sphere(b_r)) => {
                            if let Some(collision) = sphere_to_sphere((*a, ta, ra, *a_r), (*b, tb, rb, *b_r)) {
                                collisions.push(collision);
                            }
                        }
                    }
                }
            }
        }

        for collision in collisions.iter() {
            let mut a = entities.query_one::<&mut Transform>(collision.entity_a).unwrap();
            let mut b = entities.query_one::<&mut Transform>(collision.entity_b).unwrap();

            a.get().unwrap().position += collision.move_a;
            b.get().unwrap().position += collision.move_b;

            debug!("{} {} {:?} {:?}", collision.entity_a.id(), collision.entity_b.id(), collision.move_a, collision.move_b);
        }
    }
}
