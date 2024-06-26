use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{asset_library::AssetLibrary, ecs::System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub meshes_and_materials: Vec<(Uuid, Uuid)>
}

impl Model {
    pub fn new(name: String) -> Model {
        Model {
            name,
            meshes_and_materials: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelComponent {
    pub model_uuid: Uuid,
    model_name: String
}

impl ModelComponent {
    pub fn new(name: &str) -> ModelComponent {
        ModelComponent {
            model_uuid: Uuid::nil(),
            model_name: name.to_string()
        }
    }

    pub fn load_uuid(&mut self, assets: &AssetLibrary) {
        self.model_uuid = *assets.models.iter().find(|(_, v)| v.name == self.model_name).expect("Model name not found").0;
    }
}

pub struct ModelComponentUuidLoader {}

impl System for ModelComponentUuidLoader {
    fn on_start(&self, world: &crate::ecs::World, assets: &mut AssetLibrary, _state: &mut crate::state::State) {
        for (_, model_component) in world.entities.query::<&mut ModelComponent>().iter() {
            model_component.load_uuid(assets);
        }
    }

    fn on_update(&self, _world: &crate::ecs::World, _assets: &mut AssetLibrary, _state: &mut crate::state::State) {}
}
