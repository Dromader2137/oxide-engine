use vulkano::{format::Format, image::SampleCount, render_pass::{AttachmentDescription, AttachmentLoadOp, AttachmentStoreOp}};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
    RGB16,
    F32, 
    Display
}

impl TextureFormat {
    pub fn into_vulkan_foramt(&self, display_format: Format) -> Format {
        match self {
            TextureFormat::Display => display_format,
            TextureFormat::F32 => Format::D32_SFLOAT,
            TextureFormat::RGB8 => Format::R8G8B8_UNORM,
            TextureFormat::RGBA8 => Format::R8G8B8A8_UNORM,
            TextureFormat::RGB16 => Format::R8G8B8_SNORM
        }
    }
}

pub struct ColorNode {}
pub struct OutputNode {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeIO {
    Texture(TextureFormat),
}

pub trait RendererNode {
    fn get_outputs(&self) -> Vec<u32> { vec![] }
    fn get_inputs(&self) -> Vec<u32> { vec![] }
    fn get_attachments(&self) -> Vec<TextureFormat> { vec![] }
    fn get_color_attachment(&self) -> Option<u32> { None }
    fn get_depth_attachment(&self) -> Option<u32> { None }
    fn is_output(&self) -> bool { false }
}

impl RendererNode for OutputNode {
    fn get_inputs(&self) -> Vec<u32> {
        vec![
            0
        ]
    }

    fn get_attachments(&self) -> Vec<TextureFormat> {
        vec![
            TextureFormat::Display
        ]
    }
}

impl RendererNode for ColorNode {
    fn get_outputs(&self) -> Vec<u32> {
        vec![
            0
        ]
    }

    fn get_attachments(&self) -> Vec<TextureFormat> {
        vec![
            TextureFormat::Display,
            TextureFormat::F32
        ]
    }

    fn get_color_attachment(&self) -> Option<u32> {
        Some(0)
    }

    fn get_depth_attachment(&self) -> Option<u32> {
        Some(1)
    }

    fn is_output(&self) -> bool {
        true
    }
}
