use std::{sync::Arc};

use image::{io::Reader, RgbaImage};

use log::debug;
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
        debug!("{}", self.name);
        let img = Reader::open(format!("assets/textures/{}", self.name)).unwrap().decode().unwrap().into_rgba8();

        self.image = Some(Image::new(
            renderer.memeory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [img.width(), img.height(), 1],
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
            img.into_flat_samples().as_slice().to_owned(),
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
                SamplerCreateInfo::simple_repeat_linear()
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

pub struct DefaultTextureLoader {}

fn default_texture(renderer: &Renderer) -> Texture {
    let img = RgbaImage::new(1, 1);

    let image = Some(Image::new(
            renderer.memeory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [img.width(), img.height(), 1],
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
        img.into_flat_samples().as_slice().to_owned(),
    ).unwrap();

    builder
        .copy_buffer_to_image(
            CopyBufferToImageInfo::buffer_image(temp_buffer, image.as_ref().unwrap().to_owned())
        )
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = now(renderer.device.clone())
        .then_execute(renderer.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let image_view = Some(
        ImageView::new(
            image.as_ref().unwrap().clone(),
            ImageViewCreateInfo::from_image(image.as_ref().unwrap().as_ref())
        ).unwrap()
    );

    let sampler = Some(
        Sampler::new(
            renderer.device.clone(), 
            SamplerCreateInfo::default()
        ).unwrap()
    );

    Texture {
        name: "default".to_string(),
        image,
        image_view,
        sampler
    }
}

impl System for DefaultTextureLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        if assets.textures.iter().find(|x| x.name == "default".to_string()).is_none() {
            assets.textures.push(default_texture(&state.renderer));
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
