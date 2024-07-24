use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use std::usize;

use bytemuck::{Pod, Zeroable};

use log::{debug, error};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, DrawIndirectCommand,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::
    Device
;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{
    AttachmentBlend, ColorBlendAttachmentState, ColorBlendState, ColorComponents,
};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{PolygonMode, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition, VertexInputState};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::swapchain::{
    self, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    SwapchainPresentInfo,
};
use vulkano::sync::future::{FenceSignalFuture, JoinFuture};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};

use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;

use crate::asset_library::AssetLibrary;
use crate::ecs::{System, World};
use crate::state::State;
use crate::types::camera::Camera;
use crate::types::matrices::*;
use crate::types::mesh::{DynamicMesh, DynamicMeshRenderingComponent};
use crate::types::model::ModelComponent;
use crate::types::shader::{Shader, ShaderType};
use crate::types::transform::{ModelData, Transform};
use crate::types::vectors::*;
use crate::ui::ui_layout::UiVertexData;
use crate::ui::ui_rendering::UiRenderingComponent;
use crate::vulkan::context::VulkanContext;
use crate::vulkan::memory::MemoryAllocators;

use self::rendering_component::RenderingComponent;

pub mod rendering_component;

#[derive(Pod, Zeroable, Clone, Copy, Debug, Serialize, Deserialize, Vertex)]
#[repr(C)]
pub struct VertexData {
    #[format(R32G32B32A32_SFLOAT)]
    pub position: Vec3f,
    #[format(R32G32B32A32_SFLOAT)]
    pub uv: Vec2f,
    #[format(R32G32B32A32_SFLOAT)]
    pub normal: Vec3f,
    #[format(R32G32B32A32_SFLOAT)]
    pub tangent: Vec4f,
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct VPData {
    pub view: Matrix4f,
    pub projection: Matrix4f,
}

#[derive(Clone, Debug)]
pub struct Window {
    pub window_handle: Arc<winit::window::Window>,
}

impl Window {
    pub fn new(event_loop: &EventLoop) -> Window {
        Window {
            window_handle: Arc::new(WindowBuilder::new().build(&event_loop.event_loop).unwrap()),
        }
    }
}

pub struct EventLoop {
    pub event_loop: winit::event_loop::EventLoop<()>,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop {
            event_loop: winit::event_loop::EventLoop::new().unwrap(),
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

type Fence = Option<
    Arc<
        FenceSignalFuture<
            PresentFuture<
                CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>,
            >,
        >,
    >,
>;


#[allow(dead_code)]
pub struct Renderer {
    pub render_pass: Arc<RenderPass>,
    pub swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pub viewport: Viewport,

    pub vp_data: VPData,
    pub vp_pos: Vec3d,
    pub vp_buffers: Vec<Subbuffer<VPData>>,

    pub window_resized: bool,
    pub recreate_swapchain: bool,
    pub frames_in_flight: usize,

    pub fences: Vec<Fence>,
    pub mesh_cache: Vec<Vec<(Arc<Subbuffer<[VertexData]>>, Arc<Subbuffer<[u32]>>)>>,
    pub previous_fence: usize,

    pub pipelines: HashMap<(Uuid, Uuid), Arc<GraphicsPipeline>>,
    pub rendering_components: Vec<Box<dyn RenderingComponent>>,

    pub anisotropic: Option<f32>
}

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            inter: {
                format: swapchain.image_format(),
                samples: SampleCount::Sample8,
                load_op: Clear,
                store_op: Store,
            },
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D32_SFLOAT,
                samples: SampleCount::Sample8,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [inter],
            color_resolve: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap()
}

fn get_framebuffers(
    device: Arc<Device>,
    images: &Vec<Arc<Image>>,
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample8,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            let inter = ImageView::new_default(
                Image::new(
                    memory_allocator.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: image.format(),
                        extent: image.extent(),
                        usage: ImageUsage::COLOR_ATTACHMENT,
                        samples: SampleCount::Sample8,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![inter, view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

pub fn get_pipeline(state: &State, vs: &Shader, fs: &Shader, polygon_mode: PolygonMode) -> Arc<GraphicsPipeline> {
    let vertex_type = vs.shader_type;

    let vs = vs.module.as_ref().unwrap().entry_point("main").unwrap();
    let fs = fs.module.as_ref().unwrap().entry_point("main").unwrap();
    
    let vertex_input = match vertex_type {
        ShaderType::Vertex => {
            VertexInputState::new()
        },
        ShaderType::UiVertex => {
            UiVertexData::per_vertex().definition(&vs.info().input_interface).unwrap()
        },
        _ => panic!("")
    };

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        state.vulkan_context.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(state.vulkan_context.device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(state.renderer.render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        state.vulkan_context.device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [state.renderer.viewport.clone()].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState {
                polygon_mode,
                ..Default::default()
            }),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Greater,
                }),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState {
                rasterization_samples: SampleCount::Sample8,
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    color_write_mask: ColorComponents::all(),
                    color_write_enable: true,
                },
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

fn get_command_buffers(
    world: &World,
    assets: &mut AssetLibrary,
    state: &mut State,
    image_id: usize,
) -> Arc<PrimaryAutoCommandBuffer> {
    let framebuffer = state.renderer.framebuffers.get(image_id).unwrap();
    let mut builder = AutoCommandBufferBuilder::primary(
        state.memory_allocators.command_buffer_allocator.as_ref(),
        state.vulkan_context.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.0, 0.0, 0.0, 1.0].into()),
                    Some([0.0, 0.0, 0.0, 1.0].into()),
                    Some(0f32.into()),
                ],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )
        .unwrap();

    for rendering_component in state.renderer.rendering_components.iter() {
        builder = rendering_component.render(builder, world, assets, state, image_id);
    }

    builder.end_render_pass(Default::default()).unwrap();
    builder.build().unwrap()
}

fn get_swapchain(
    window_size: PhysicalSize<u32>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = window_size;
    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap()
}

fn recreate_pipelines(assets: &AssetLibrary, state: &mut State) {
    for (_, material) in assets.materials.iter() {
        state.renderer.pipelines.insert(
            (material.vertex_shader, material.fragment_shader),
            get_pipeline(
                state, 
                assets.shaders.get(&material.vertex_shader).unwrap(), 
                assets.shaders.get(&material.fragment_shader).unwrap(), 
                material.rendering_type.into()
            )        
        );
    }
}

fn recalculate_projection(world: &World, state: &mut State, new_dimensions: PhysicalSize<u32>) {
    let mut camera = world.entities.query::<&Camera>();
    let camera_data = camera.iter().next().expect("Camera not found").1;
    state.renderer.vp_data.projection = Matrix4f::perspective(
        camera_data.vfov.to_radians(),
        (new_dimensions.width as f32) / (new_dimensions.height as f32),
        camera_data.near
    );
}

fn handle_possible_resize(world: &World, assets: &AssetLibrary, state: &mut State) {
    if state.renderer.window_resized || state.renderer.recreate_swapchain {
        state.renderer.recreate_swapchain = false;
        state.renderer.window_resized = false;

        let new_dimensions = state.window.window_handle.inner_size();
        let (new_swapchain, new_images) = state
            .renderer
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: new_dimensions.into(),
                ..state.renderer.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        state.renderer.swapchain = new_swapchain;
        state.renderer.images = new_images;
        state.renderer.framebuffers = get_framebuffers(
            state.vulkan_context.device.clone(),
            &state.renderer.images,
            state.renderer.render_pass.clone(),
        );

        state.renderer.viewport.extent = new_dimensions.into();

        recalculate_projection(world, state, new_dimensions);
        recreate_pipelines(assets, state);
    }
}

#[allow(clippy::arc_with_non_send_sync)]
fn render(world: &World, assets: &mut AssetLibrary, state: &mut State) {
    let timer = Instant::now();
    let (image_i, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(state.renderer.swapchain.clone(), None)
            .map_err(Validated::unwrap)
        {
            Ok(r) => r,
            Err(VulkanError::OutOfDate) => {
                state.renderer.recreate_swapchain = true;
                return;
            }
            Err(e) => panic!("failed to acquire next image: {e}"),
        };

    if suboptimal {
        state.renderer.recreate_swapchain = true;
    }
    
    let command_buffer = get_command_buffers(world, assets, state, image_i as usize);
    
    let previous_future = match state.renderer.fences[state.renderer.previous_fence].clone() {
        None => {
            let mut now = sync::now(state.vulkan_context.device.clone());
            now.cleanup_finished();
            now.boxed()
        }
        Some(fence) => fence.boxed(),
    };
    
    if let Some(image_fence) = &state.renderer.fences[image_i as usize] {
        image_fence.wait(None).unwrap();
    }

    {
        let mut contents = state
            .renderer
            .vp_buffers
            .get(image_i as usize)
            .unwrap()
            .write()
            .unwrap();
        *contents = state.renderer.vp_data;
        drop(contents);
    }
    
    let future = previous_future
        .join(acquire_future)
        .then_execute(state.vulkan_context.queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(
            state.vulkan_context.queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(state.renderer.swapchain.clone(), image_i),
        )
        .then_signal_fence_and_flush();
    
    state.renderer.fences[image_i as usize] = match future.map_err(Validated::unwrap) {
        Ok(value) => Some(Arc::new(value)),
        Err(VulkanError::OutOfDate) => {
            state.renderer.recreate_swapchain = true;
            None
        }
        Err(e) => {
            error!("Failed to flush future: {e}");
            None
        }
    };
    state.renderer.previous_fence = image_i as usize;
    debug!(" Inside render: {}", timer.elapsed().as_millis());
}

impl Renderer {
    pub fn new(context: &VulkanContext, memory_allocators: &MemoryAllocators, window: &Window) -> Renderer {
        let (swapchain, images) = get_swapchain(
            window.window_handle.inner_size(),
            context.physical_device.clone(),
            context.device.clone(),
            context.render_surface.clone(),
        );


        let render_pass = get_render_pass(context.device.clone(), swapchain.clone());
        let framebuffers = get_framebuffers(context.device.clone(), &images, render_pass.clone());

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.window_handle.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let frames_in_flight = images.len();
        let fences = vec![None; frames_in_flight];

        let vp_buffers = {
            let mut vec = Vec::new();
            for _ in 0..frames_in_flight {
                vec.push(
                    Buffer::new_sized::<VPData>(
                        memory_allocators.standard_memory_allocator.clone(),
                        BufferCreateInfo {
                            usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                )
            }
            vec
        };
        let vp_data = VPData {
            view: Matrix4f::indentity(),
            projection: Matrix4f::indentity(),
        };
        let vp_pos = Vec3d::new([0.0, 0.0, 0.0]);

        Renderer {
            render_pass,
            swapchain,
            images,
            framebuffers,
            viewport,
            window_resized: false,
            recreate_swapchain: false,
            frames_in_flight: 0,
            fences,
            previous_fence: 0,
            vp_data,
            vp_pos,
            vp_buffers,
            pipelines: HashMap::new(),
            rendering_components: vec![
                Box::new(DynamicMeshRenderingComponent { dynamic_mesh_data: RefCell::new(HashMap::new()) }),
                Box::new(UiRenderingComponent {})
            ],
            mesh_cache: vec![Vec::new(); frames_in_flight],
            anisotropic: Some(context.physical_device.properties().max_sampler_anisotropy)
        }
    }
}

pub struct RendererHandler {}

impl System for RendererHandler {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, world: &World, assets: &mut AssetLibrary, state: &mut State) {
        let timer = Instant::now();
        handle_possible_resize(world, assets, state);
        debug!(" Resize: {}", timer.elapsed().as_millis());
        let timer = Instant::now();
        render(world, assets, state);
        debug!(" Render: {}", timer.elapsed().as_millis());
    }
}
