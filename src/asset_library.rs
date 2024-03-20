use crate::types::{mesh::Mesh, shader::Shader};

pub struct AssetLibrary {
    pub meshes: Vec<Mesh>,
    pub shaders: Vec<Shader>,
}
