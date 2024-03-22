use std::sync::Arc;

use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{get_pipeline, Renderer}, state::State, utility::read_file_to_words};

#[derive(Debug)]
pub enum ShaderType {
    Fragment,
    Vertex,
}

#[derive(Debug)]
pub struct Shader {
    pub name: String,
    pub shader_type: ShaderType,
    pub source: Vec<u32>,
    pub module: Option<Arc<ShaderModule>>,
}

impl Shader {
    pub fn load(&mut self, renderer: &mut Renderer) {
        unsafe {
            self.module = Some(ShaderModule::new(
                renderer.device.as_ref().unwrap().clone(), 
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

        let fragment_shaders = assets.shaders.iter()
            .filter(|x| matches!(x.shader_type, ShaderType::Fragment));
        let vertex_shaders = assets.shaders.iter()
            .filter(|x| matches!(x.shader_type, ShaderType::Vertex));
        
        for frag in fragment_shaders {
            for vert in vertex_shaders.clone() {
                state.renderer.pipelines.insert(
                    (vert.name.clone(), frag.name.clone()),
                    get_pipeline(state, vert, frag)
                );
            }
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
