use crate::types::{material::Material, mesh::Mesh, shader::Shader, texture::Texture};

pub struct AssetLibrary {
    pub meshes: Vec<Mesh>,
    pub shaders: Vec<Shader>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>
}
