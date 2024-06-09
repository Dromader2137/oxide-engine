use crate::types::{material::Material, model::Model, shader::Shader, texture::Texture};

pub struct AssetLibrary {
    pub shaders: Vec<Shader>,
    pub textures: Vec<Texture>,
    pub models: Vec<Model>,
    pub materials: Vec<Material>
}
