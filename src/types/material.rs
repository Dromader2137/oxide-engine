use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, state::State};

use super::vectors::Vec3f;

#[derive(BufferContents, Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct MaterialParameters {
    pub diffuse_color: Vec3f,
    pub roughness: f32,
    pub use_roughness_texture: u8,
    pub use_diffuse_texture: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attachment {
    Texture(String)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub attachments: Vec<Attachment>,
    pub parameters: Option<MaterialParameters>,
    #[serde(skip)]
    pub parameter_buffer: Option<Subbuffer<MaterialParameters>>
}

impl Material {
    pub fn new(
        name: &str,
        vertex_shader: &'static str,
        fragment_shader: &'static str,
        attachments: Vec<Attachment>,
        parameters: Option<MaterialParameters>
    ) -> Material {
        Material {
            name: name.to_string(),
            vertex_shader: vertex_shader.to_string(),
            fragment_shader: fragment_shader.to_string(),
            attachments,
            parameters,
            parameter_buffer: None
        }
    }

    pub fn load(&mut self, state: &State) {
        if self.parameters.is_none() { return; }

        self.parameter_buffer = Some(
            Buffer::new_sized::<MaterialParameters>(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                }
            ).unwrap()
        );
        let mut content = self.parameter_buffer.as_ref().unwrap().write().unwrap();
        *content = self.parameters.as_ref().unwrap().clone();
    }
}

pub struct MaterialLoader {}

impl System for MaterialLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for material in assets.materials.iter_mut() {
            material.load(state);
        }
    }

    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
