use std::collections::HashMap;

use crate::{asset_library::AssetLibrary, types::{material::{Attachment, Material, MaterialParameters, RenderingType}, model::Model, shader::{Shader, ShaderType}, texture::Texture, vectors::Vec2f}, ui::ui_layout::{Anchor, UiElement, UiElementType, UiElements}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderDescription {
    pub name: String,
    pub shader_type: ShaderType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureDescription {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDescription {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AttachmentDescription {
    Texture(String),
    DefaultTexture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialDescription {
    pub name: String,
    pub vertex: String,
    pub fragment: String,
    pub attachments: Vec<AttachmentDescription>,
    pub paramaters: Option<MaterialParameters>,
    pub rendering_type: RenderingType
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiElementDescription {
    pub element_type: UiElementType,
    pub name: String,
    pub material: String,
    pub position: Vec2f,
    pub screen_anchor: Anchor,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetDescriptions {
    pub shaders: Vec<ShaderDescription>,
    pub textures: Vec<TextureDescription>,
    pub models: Vec<ModelDescription>,
    pub materials: Vec<MaterialDescription>,
    pub ui_layouts: Vec<UiElementDescription>
}

impl AssetDescriptions {
    pub fn generate_library(&self) -> AssetLibrary {
        let shaders: HashMap<Uuid, Shader> = {
            let mut map = HashMap::new();
            for shader_description in self.shaders.iter() {
                map.insert(Uuid::new_v4(), Shader::new(shader_description.name.clone(), shader_description.shader_type));
            }
            map
        };
        
        let textures: HashMap<Uuid, Texture> = {
            let mut map = HashMap::new();
            for texture_description in self.textures.iter() {
                map.insert(Uuid::new_v4(), Texture::new(texture_description.name.clone()));
            }
            map
        };
        
        let models: HashMap<Uuid, Model> = {
            let mut map = HashMap::new();
            for model_description in self.models.iter() {
                map.insert(Uuid::new_v4(), Model::new(model_description.name.clone()));
            }
            map
        };

        let materials: HashMap<Uuid, Material> = {
            let mut map = HashMap::new();
            for material_description in self.materials.iter() {
                let vertex_uuid = shaders.iter().find(|(_, shader)| shader.name == material_description.vertex)
                    .expect("Vertex shader not found").0;
                let fragment_uuid = shaders.iter().find(|(_, shader)| shader.name == material_description.fragment)
                    .expect("Vertex shader not found").0;
                map.insert(Uuid::new_v4(), Material::new(
                    material_description.name.clone(), 
                    *vertex_uuid, 
                    *fragment_uuid, 
                    material_description.attachments.iter().map(|x|
                        match x {
                            AttachmentDescription::Texture(name) => {
                                let texture_uuid = *textures.iter().find(|(_, v)| v.name == *name).expect("Textre not found").0;
                                Attachment::Texture(texture_uuid)
                            },
                            AttachmentDescription::DefaultTexture => Attachment::DefaultTexture
                        }
                    ).collect(),
                    material_description.paramaters.clone(),
                    material_description.rendering_type
                )
                );
            }
            map
        };
        
        let ui: UiElements = {
            let mut vec = Vec::new();
            for ui_element_desc in self.ui_layouts.iter() {
                let material_uuid = materials.iter().find(|(_, material)| material.name == ui_element_desc.material)
                    .expect("Material not found").0;
                vec.push(
                    UiElement::new(&ui_element_desc.name, ui_element_desc.element_type, *material_uuid, ui_element_desc.screen_anchor, ui_element_desc.position, ui_element_desc.width, ui_element_desc.height)
                );
            }
            UiElements { elements: vec }
        };

        AssetLibrary {
            shaders,
            textures,
            models,
            materials,
            meshes: HashMap::new(),
            ui
        }
    }
}
