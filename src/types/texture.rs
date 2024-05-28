use std::{fs::File, sync::Arc, io::{Cursor, Read}};

use log::debug;
use png::ColorType;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo}, format::Format, image::{sampler::{Sampler, SamplerCreateInfo}, view::{ImageView, ImageViewCreateInfo}, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}, sync::{now, GpuFuture}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::Renderer, state::State};

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub image: Option<Arc<Image>>,
    pub image_view: Option<Arc<ImageView>>,
    pub sampler: Option<Arc<Sampler>>
}

impl Texture {
    pub fn new(name: String) -> Texture {
        Texture { 
            name, 
            image: None,
            image_view: None, 
            sampler: None
        }
    }

    fn load(&mut self, renderer: &mut Renderer) {
        let (image_data, image_dimensions, format) = {
            let mut file = File::open(format!("assets/textures/{}.png", self.name))
                .unwrap();
            let mut png_bytes: Vec<u8> = Vec::new();
            file.read_to_end(&mut png_bytes).unwrap();
            
            let cursor = Cursor::new(png_bytes);
            let decoder = png::Decoder::new(cursor);
            let mut reader = decoder.read_info().unwrap();
            let info = reader.info().clone();
            debug!("{:?}", info);
            let mut image_data = Vec::new();
            let depth: u32 = match info.bit_depth {
                png::BitDepth::One => 1,
                png::BitDepth::Two => 2,
                png::BitDepth::Four => 4,
                png::BitDepth::Eight => 8,
                png::BitDepth::Sixteen => 16,
            };
            image_data.resize((info.width * info.height * depth) as usize, 0);
            reader.next_frame(&mut image_data).unwrap();
            (image_data, [info.width, info.height, 1], info.color_type)
        };

        self.image = Some(Image::new(
            renderer.memeory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: match format {
                    ColorType::Rgba => Format::R8G8B8A8_UNORM,
                    ColorType::Rgb => Format::R8G8B8_UNORM,
                    _ => Format::R8_UNORM
                },
                extent: image_dimensions,
                usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        ).unwrap());
        
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            renderer.device.clone(),
            Default::default(),
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            renderer.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let temp_buffer = Buffer::from_iter(
            renderer.memeory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            image_data,
        ).unwrap();

        builder
            .copy_buffer_to_image(
                CopyBufferToImageInfo::buffer_image(temp_buffer, self.image.as_ref().unwrap().to_owned())
            )
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = now(renderer.device.clone())
            .then_execute(renderer.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        self.image_view = Some(
            ImageView::new(
                self.image.as_ref().unwrap().clone(),
                ImageViewCreateInfo::from_image(self.image.as_ref().unwrap().as_ref())
            ).unwrap()
        );

        self.sampler = Some(
            Sampler::new(
                renderer.device.clone(), 
                SamplerCreateInfo::default()
            ).unwrap()
        );
    }
}

pub struct TextureLoader {}

impl System for TextureLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for mesh in assets.textures.iter_mut() {
            mesh.load(&mut state.renderer);
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
