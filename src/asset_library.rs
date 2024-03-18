use crate::rendering::{Mesh, Renderer, Shader};

pub struct AssetLibrary {
    pub meshes: Vec<Mesh>,
    pub shaders: Vec<Shader>,
    pub renderer: Renderer
}
