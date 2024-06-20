use crate::{asset_library::AssetLibrary, types::{material::{Attachment, Material, MaterialParameters}, model::Model, shader::{Shader, ShaderType}, texture::Texture}};
use serde::{Deserialize, Serialize};

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
pub struct MaterialDescription {
    pub name: String,
    pub vertex: String,
    pub fragment: String,
    pub paramaters: Option<MaterialParameters>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetDescriptions {
    pub shaders: Vec<ShaderDescription>,
    pub textures: Vec<TextureDescription>,
    pub models: Vec<ModelDescription>,
    pub materials: Vec<MaterialDescription>,
}

impl AssetDescriptions {
    pub fn generate_library(&self) -> AssetLibrary {
        AssetLibrary {
            shaders: {
                self.shaders.iter().map(|x| Shader::new(x.name.clone(), x.shader_type)).collect()
            },
            textures: {
                self.textures.iter().map(|x| Texture::new(x.name.clone())).collect()
            },
            models: {
                self.models.iter().map(|x| Model::new(x.name.clone())).collect()
            },
            materials: {
                self.materials.iter().map(|x| Material::new(
                        x.name.clone(), 
                        x.vertex.clone(),
                        x.fragment.clone(),
                        vec![Attachment::Texture("default".to_string()), Attachment::Texture("default".to_string())], 
                        x.paramaters.clone()
                    )
                ).collect()
            }, 
            meshes: vec![]
        }
    }
}
