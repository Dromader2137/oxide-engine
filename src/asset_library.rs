use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{types::{material::Material, mesh::Mesh, model::Model, shader::Shader, texture::Texture}, ui::ui_layout::UiElement};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetLibrary {
    pub shaders: HashMap<Uuid, Shader>,
    pub textures: HashMap<Uuid, Texture>,
    pub models: HashMap<Uuid, Model>,
    pub materials: HashMap<Uuid, Material>,
    pub meshes: HashMap<Uuid, Mesh>,
    pub ui: HashMap<Uuid, UiElement>,
}
