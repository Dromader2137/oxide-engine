use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub meshes_and_materials: Vec<(String, String)>
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
    pub model_name: String
}
