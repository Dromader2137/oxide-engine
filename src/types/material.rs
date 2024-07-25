use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use vulkano::{buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, pipeline::graphics::rasterization::PolygonMode};
use uuid::Uuid;

use crate::{asset_library::AssetLibrary, ecs::{System, World}, state::State};

use super::vectors::Vec3f;

#[derive(BufferContents, Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct MaterialParameters {
    pub diffuse_color: Vec3f,
    pub use_diffuse_texture: u32,
    pub use_normal_texture: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Attachment {
    DefaultTexture,
    Texture(Uuid)
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderingType {
    Fill,
    Line,
    Point
}

impl From<RenderingType> for PolygonMode {
    fn from(val: RenderingType) -> Self {
        match val {
            RenderingType::Fill => PolygonMode::Fill,
            RenderingType::Line => PolygonMode::Line,
            RenderingType::Point => PolygonMode::Point
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub vertex_shader: Uuid,
    pub fragment_shader: Uuid,
    pub attachments: Vec<Attachment>,
    pub parameters: Option<MaterialParameters>,
    pub rendering_type: RenderingType,
    #[serde(skip)]
    pub parameter_buffer: Option<Subbuffer<MaterialParameters>>,
}

impl Material {
    pub fn new(
        name: String,
        vertex_shader: Uuid,
        fragment_shader: Uuid,
        attachments: Vec<Attachment>,
        parameters: Option<MaterialParameters>,
        rendering_type: RenderingType,
    ) -> Material {
        Material {
            name: name.to_string(),
            vertex_shader,
            fragment_shader,
            attachments,
            parameters,
            parameter_buffer: None,
            rendering_type
        }
    }

    pub fn load(&mut self, state: &State) {
        if self.parameters.is_none() { return; }

        self.parameter_buffer = Some(
            Buffer::new_sized::<MaterialParameters>(
                state.memory_allocators.standard_memory_allocator.clone(),
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
        for (_, material) in assets.materials.iter_mut() {
            material.load(state);
        }
    }

    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
