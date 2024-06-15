use serde::{Deserialize, Serialize};

use crate::types::{material::Material, mesh::Mesh, model::Model, shader::Shader, texture::Texture};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetLibrary {
    pub shaders: Vec<Shader>,
    pub textures: Vec<Texture>,
    pub models: Vec<Model>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
}

