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
        let mut query =
            entities.query::<(&Transform, &Collider)>();
        let vec = query.iter().map(|(_, x)| x).collect::<Vec<_>>();

        for (i, (t1, c1)) in vec.iter().enumerate() {
            for (j, (t2, c2)) in vec.iter().enumerate() {
                if i <= j { continue; }

                match c1 {
                    Collider::Sphere(a_r) => {
                        match c2 {
                            Collider::Sphere(b_r) => {
                                sphere_to_sphere(*a_r, t1, *b_r, t2);
                            }
                        }
                    }
                }

            }
        }
    }
}
