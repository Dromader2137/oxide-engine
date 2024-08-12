use hecs::Entity;
use log::debug;

use crate::{ecs::System, types::transform::Transform};

use super::collider::{sphere_to_sphere, Collider};

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
            let mut query = entities.query::<(&Transform, &Collider)>();
            let vec = query.iter().map(|(e, x)| (e, x)).collect::<Vec<_>>();

            for (a, (t1, c1)) in vec.iter() {
                for (b, (t2, c2)) in vec.iter() {
                    if a <= b { continue; }

                    match (c1, c2) {
                        (Collider::Sphere(a_r), Collider::Sphere(b_r)) => {
                            if let Some(collision) = sphere_to_sphere(*a, *a_r, t1, *b, *b_r, t2) {
                                collisions.push(collision);
                            }
                        }
                    }
                }
            }
        }

        for collision in collisions.iter() {
            // let a = entities.query_one::<(&Transform)>(collision.entity_a).unwrap();
            // let b = entities.query_one::<(&Transform)>(collision.entity_b).unwrap();

            debug!("{} {}", collision.entity_a.id(), collision.entity_b.id());
        }
    }
}
