use std::{fs::File, io::Read, sync::Arc};

use serde::{Deserialize, Serialize};
use vulkano::shader::{spirv::bytes_to_words, ShaderModule, ShaderModuleCreateInfo};
use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::{get_pipeline, PipelineIdentifier}, state::State, vulkan::context::VulkanContext};

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
    pub fn load(&mut self, context: &VulkanContext) {
        unsafe {
            self.module = Some(ShaderModule::new(
                context.device.clone(), 
                ShaderModuleCreateInfo::new(self.source.as_slice())
            ).unwrap());
        }
    }

    pub fn new(name: String, shader_type: ShaderType) -> Shader {
        Shader {
            name: name.clone(),
            shader_type,
            source: read_file_to_words(format!("assets/shaders/bin/{}.spv", name).as_str()),
            module: None
        }
    }
}

pub fn read_file_to_words(path: &str) -> Vec<u32> {
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; file.metadata().unwrap().len() as usize];
    file.read_exact(buffer.as_mut_slice()).unwrap();
    bytes_to_words(buffer.as_slice()).unwrap().to_vec()
}

pub struct ShaderLoader {}

impl System for ShaderLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for (_, shader) in assets.shaders.iter_mut() {
            shader.load(&state.vulkan_context);
        }

        for (_, material) in assets.materials.iter() {
            state.renderer.pipelines.insert(
                PipelineIdentifier::new(material.vertex_shader, material.fragment_shader, material.rendering_type),
                get_pipeline(
                    state, 
                    assets.shaders.get(&material.vertex_shader).unwrap(), 
                    assets.shaders.get(&material.fragment_shader).unwrap(), 
                    material.rendering_type.into()
                )        
            );
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
