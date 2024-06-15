use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{get_pipeline, Renderer}, state::State, utility::read_file_to_words};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ShaderType {
    Fragment,
    Vertex,
    UiFragment,
    UiVertex
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shader {
    pub name: String,
    pub shader_type: ShaderType,
    pub source: Vec<u32>,
    #[serde(skip)]
    pub module: Option<Arc<ShaderModule>>,
}

impl Shader {
    pub fn load(&mut self, renderer: &mut Renderer) {
        unsafe {
            self.module = Some(ShaderModule::new(
                renderer.device.clone(), 
                ShaderModuleCreateInfo::new(self.source.as_slice())
            ).unwrap());
        }
    }

    pub fn new(name: String, shader_type: ShaderType) -> Shader {
        Shader {
            name: name.clone(),
            shader_type,
            source: read_file_to_words(format!("shaders/bin/{}.spv", name).as_str()),
            module: None
        }
    }
}

pub struct ShaderLoader {}

impl System for ShaderLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for shader in assets.shaders.iter_mut() {
            shader.load(&mut state.renderer);
        }

        for material in assets.materials.iter() {
            state.renderer.pipelines.insert(
                (material.vertex_shader.clone(), material.fragment_shader.clone()),
                get_pipeline(
                    state, 
                    assets.shaders.iter().find(|x| x.name == material.vertex_shader).unwrap(), 
                    assets.shaders.iter().find(|x| x.name == material.fragment_shader).unwrap()
                )        
            );
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
