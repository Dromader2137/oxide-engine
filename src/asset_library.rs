use crate::types::{material::Material, shader::Shader, texture::Texture};

pub struct AssetLibrary {
    pub shaders: Vec<Shader>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>
}
