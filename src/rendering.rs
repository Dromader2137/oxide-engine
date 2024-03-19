use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, CopyBufferInfo,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{
    self, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    SwapchainPresentInfo,
};
use vulkano::sync::future::{FenceSignalFuture, JoinFuture};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use winit::window::WindowBuilder;

use crate::asset_library::AssetLibrary;
use crate::ecs::{System, World};
use crate::state::State;
use crate::types::buffers::*;
use crate::types::matrices::*;
use crate::types::static_mesh::StaticMesh;
use crate::types::transform::Transform;
use crate::types::vectors::*;
use crate::utility::read_file_to_words;

#[derive(BufferContents, Vertex, Clone, Copy, Debug)]
#[repr(C)]
pub struct VertexData {
    #[format(R32G32B32_SFLOAT)]
    pub position: Vec3f,
    #[format(R32G32_SFLOAT)]
    pub uv: Vec2f,
    #[format(R32G32B32_SFLOAT)]
    pub normal: Vec3f,
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct VPData {
    pub view: Matrix4f,
    pub projection: Matrix4f,
}

#[derive(Clone, Copy)]
pub struct Camera {
    pub vfov: f32,
    pub near: f32,
    pub far: f32,
}

pub struct CameraUpdater {}

impl System for CameraUpdater {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        let mut camera = world.borrow_component_vec_mut::<Camera>().unwrap();
        let mut transform = world.borrow_component_vec_mut::<Transform>().unwrap();
        let zip = camera.iter_mut().zip(transform.iter_mut());
        let mut iter =
            zip.filter_map(|(camera, transform)| Some((camera.as_mut()?, transform.as_mut()?)));
        let (_, transform_data) = iter.next().unwrap();
        let cam_rot = Matrix4f::rotation_xzy(transform_data.rotation);
        state.renderer.vp_data.view = Matrix4f::look_at(
            transform_data.position.to_vec3f(),
            cam_rot.vec_mul(Vec3f::new([1.0, 0.0, 0.0])),
            cam_rot.vec_mul(Vec3f::new([0.0, 1.0, 0.0])),
        );
        state
            .renderer
            .vp_buffer
            .as_mut()
            .unwrap()
            .write(state.renderer.vp_data);
    }
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ModelData {
    pub translation: Matrix4f,
}

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
    pub data: Option<Arc<ShaderModule>>,
}

impl Shader {
    pub fn load(&mut self, renderer: &mut Renderer) {
        unsafe {
            self.data = Some(ShaderModule::new(
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
            data: None
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

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub mesh: Vec<VertexData>,
    pub vertex: String,
    pub fragment: String,
    pub buffer: Option<Subbuffer<[VertexData]>>,
}

impl Mesh {
    pub fn load(&mut self, renderer: &mut Renderer) {
        self.buffer = Some(
            Buffer::from_iter(
                renderer.memeory_allocator.as_ref().unwrap().clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.mesh.clone(),
            )
            .unwrap(),
        );
    }
}

pub struct MeshLoader {}

impl System for MeshLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for mesh in assets.meshes.iter_mut() {
            mesh.load(&mut state.renderer);
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
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

#[derive(Clone)]
pub struct Renderer {
    library: Option<Arc<VulkanLibrary>>,
    instance: Option<Arc<Instance>>,
    surface: Option<Arc<Surface>>,
    physical_device: Option<Arc<PhysicalDevice>>,
    queue_family_index: Option<u32>,
    pub device: Option<Arc<Device>>,
    pub queue: Option<Arc<Queue>>,
    pub memeory_allocator: Option<Arc<StandardMemoryAllocator>>,
    pub render_pass: Option<Arc<RenderPass>>,
    pub swapchain: Option<Arc<Swapchain>>,
    pub vp_data: VPData,
    pub vp_buffer: Option<UpdatableBuffer<VPData>>,
    images: Option<Vec<Arc<Image>>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
    pub viewport: Option<Viewport>,
    pub command_buffers: Option<Vec<Arc<PrimaryAutoCommandBuffer>>>,
    pub window_resized: bool,
    pub command_buffer_outdated: bool,
    pub recreate_swapchain: bool,
    pub frames_in_flight: usize,
    pub fences: Option<
        Vec<
            Option<
                Arc<
                    FenceSignalFuture<
                        PresentFuture<
                            CommandBufferExecFuture<
                                JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
    pub previous_fence: usize,
    pipelines: HashMap<(String, String), Arc<GraphicsPipeline>>,
}

fn select_physical_device(state: &mut State, device_extensions: &DeviceExtensions) {
    let (physical_device, queue_family_index) = state
        .renderer
        .instance
        .as_ref()
        .unwrap()
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &state.renderer.surface.clone().unwrap())
                            .unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available");

    state.renderer.physical_device = Some(physical_device);
    state.renderer.queue_family_index = Some(queue_family_index);
}

fn get_render_pass(state: &mut State) {
    state.renderer.render_pass = Some(
        vulkano::single_pass_renderpass!(
        state.renderer.device.as_ref().unwrap().clone(),
        attachments: {
            inter: {
                format: state.renderer.swapchain.as_ref().unwrap().image_format(), // set the format the same as the swapchain
                samples: 8,
                load_op: Clear,
                store_op: Store,
            },
            color: {
                format: state.renderer.swapchain.as_ref().unwrap().image_format(), // set the format the same as the swapchain
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D32_SFLOAT,
                samples: 8,
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
        .unwrap(),
    )
}

fn get_framebuffers(state: &mut State) {
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(
        state.renderer.device.as_ref().unwrap().clone(),
    ));

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: state.renderer.images.as_ref().unwrap()[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample8,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    state.renderer.framebuffers = Some(
        state
            .renderer
            .images
            .as_ref()
            .unwrap()
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
                    state.renderer.render_pass.as_ref().unwrap().clone(),
                    FramebufferCreateInfo {
                        attachments: vec![inter, view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>(),
    )
}

pub fn get_pipeline(state: &State, vs: &Shader, fs: &Shader) -> Arc<GraphicsPipeline> {
    let vs = vs.data.as_ref().unwrap().entry_point("main").unwrap();
    let fs = fs.data.as_ref().unwrap().entry_point("main").unwrap();

    let vertex_input_state = VertexData::per_vertex()
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        state.renderer.device.as_ref().unwrap().clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(state.renderer.device.as_ref().unwrap().clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(state.renderer.render_pass.as_ref().unwrap().clone(), 0).unwrap();

    GraphicsPipeline::new(
        state.renderer.device.as_ref().unwrap().clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [state.renderer.viewport.as_ref().unwrap().clone()]
                    .into_iter()
                    .collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState {
                rasterization_samples: SampleCount::Sample8,
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

pub fn update_command_buffers(world: &World, assets: &AssetLibrary, state: &mut State) {
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
        state.renderer.device.as_ref().unwrap().clone(),
        Default::default(),
    );
    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        state.renderer.device.as_ref().unwrap().clone(),
        Default::default(),
    );

    state.renderer.command_buffers = Some(
        state
            .renderer
            .framebuffers
            .as_ref()
            .unwrap()
            .iter()
            .map(|framebuffer| {
                let mut meshes = world.borrow_component_vec_mut::<StaticMesh>().unwrap();
                let mut transforms = world.borrow_component_vec_mut::<Transform>().unwrap();

                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    state.renderer.queue.as_ref().unwrap().queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                )
                .unwrap();

                for transform in transforms.iter().filter(|x| x.is_some()) {
                    builder
                        .copy_buffer(CopyBufferInfo::buffers(
                            transform
                                .as_ref()
                                .unwrap()
                                .buffer
                                .as_ref()
                                .unwrap()
                                .staging_buffer
                                .clone(),
                            transform
                                .as_ref()
                                .unwrap()
                                .buffer
                                .as_ref()
                                .unwrap()
                                .main_buffer
                                .clone(),
                        ))
                        .unwrap();
                }

                builder
                    .copy_buffer(CopyBufferInfo::buffers(
                        state
                            .renderer
                            .vp_buffer
                            .as_ref()
                            .unwrap()
                            .staging_buffer
                            .clone(),
                        state
                            .renderer
                            .vp_buffer
                            .as_ref()
                            .unwrap()
                            .main_buffer
                            .clone(),
                    ))
                    .unwrap()
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.1, 0.1, 0.1, 1.0].into()),
                                Some([0.1, 0.1, 0.1, 1.0].into()),
                                Some(1f32.into()),
                            ],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap();

                let zip = meshes.iter_mut().zip(transforms.iter_mut());
                let iter = zip.filter_map(|(mesh, transform)| Some((mesh.as_mut()?, transform.as_mut()?)));

                for (static_mesh, transform) in iter {
                    let mesh = assets.meshes.iter().find(|x| x.name == static_mesh.mesh_name).unwrap();
                    let pipeline = state
                        .renderer
                        .pipelines
                        .get(&(mesh.vertex.clone(), mesh.fragment.clone()))
                        .unwrap()
                        .clone();

                    let vp_set = PersistentDescriptorSet::new(
                        &descriptor_set_allocator,
                        pipeline.layout().set_layouts().get(0).unwrap().clone(),
                        [WriteDescriptorSet::buffer(
                            0,
                            state
                                .renderer
                                .vp_buffer
                                .as_ref()
                                .unwrap()
                                .main_buffer
                                .clone(),
                        )],
                        [],
                    )
                    .unwrap();

                    let m_set = PersistentDescriptorSet::new(
                        &descriptor_set_allocator,
                        pipeline.layout().set_layouts().get(1).unwrap().clone(),
                        [WriteDescriptorSet::buffer(
                            0,
                            transform.buffer.as_ref().unwrap().staging_buffer.clone(),
                        )],
                        [],
                    )
                    .unwrap();

                    builder
                        .bind_pipeline_graphics(pipeline.clone())
                        .unwrap()
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            (vp_set.clone(), m_set.clone()),
                        )
                        .unwrap()
                        .bind_vertex_buffers(0, mesh.buffer.as_ref().unwrap().clone())
                        .unwrap()
                        .draw(mesh.mesh.len() as u32, 1, 0, 0)
                        .unwrap();
                }

                builder.end_render_pass(Default::default()).unwrap();
                builder.build().unwrap()
            })
            .collect(),
    )
}

fn get_swapchain(state: &mut State) {
    let (swapchain, images) = {
        let caps = state
            .renderer
            .physical_device
            .as_ref()
            .unwrap()
            .surface_capabilities(
                &state.renderer.surface.as_ref().unwrap(),
                Default::default(),
            )
            .expect("failed to get surface capabilities");

        let dimensions = state.window.window_handle.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = state
            .renderer
            .physical_device
            .as_ref()
            .unwrap()
            .surface_formats(
                &state.renderer.surface.as_ref().unwrap(),
                Default::default(),
            )
            .unwrap()[0]
            .0;

        Swapchain::new(
            state.renderer.device.as_ref().unwrap().clone(),
            state.renderer.surface.as_ref().unwrap().clone(),
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
    };
    state.renderer.swapchain = Some(swapchain);
    state.renderer.images = Some(images);
}

pub fn handle_possible_resize(world: &mut World, assets: &AssetLibrary, state: &mut State) {
    if state.renderer.window_resized || state.renderer.recreate_swapchain {
        state.renderer.recreate_swapchain = false;
        state.renderer.window_resized = false;

        let new_dimensions = state.window.window_handle.inner_size();

        let (new_swapchain, new_images) = state
            .renderer
            .swapchain
            .as_ref()
            .unwrap()
            .recreate(SwapchainCreateInfo {
                image_extent: new_dimensions.into(),
                ..state.renderer.swapchain.as_ref().unwrap().create_info()
            })
            .expect("failed to recreate swapchain");

        state.renderer.swapchain = Some(new_swapchain);
        state.renderer.images = Some(new_images);
        get_framebuffers(state);

        let camera = world.borrow_component_vec_mut::<Camera>().unwrap();
        let transform = world.borrow_component_vec_mut::<Transform>().unwrap();
        let zip = camera.iter().zip(transform.iter());
        let mut iter =
            zip.filter_map(|(camera, transform)| Some((camera.as_ref()?, transform.as_ref()?)));
        let (camera_data, _) = iter.next().unwrap();
        state.renderer.vp_data.projection = Matrix4f::perspective(
            camera_data.vfov.to_radians(),
            (new_dimensions.width as f32) / (new_dimensions.height as f32),
            camera_data.near,
            camera_data.far,
        );

        state.renderer.viewport.as_mut().unwrap().extent = new_dimensions.into();
        let iter: Vec<(String, String)> =
            state.renderer.pipelines.keys().map(|x| x.clone()).collect();
        for pipeline in iter.iter() {
            state.renderer.pipelines.insert(
                pipeline.clone(),
                get_pipeline(
                    state,
                    assets
                        .shaders
                        .iter()
                        .filter(|x| x.name == pipeline.0)
                        .next()
                        .unwrap(),
                    assets
                        .shaders
                        .iter()
                        .filter(|x| x.name == pipeline.1)
                        .next()
                        .unwrap(),
                ),
            );
        }

        drop(camera);
        drop(transform);
        update_command_buffers(world, assets, state);
    }
    if state.renderer.command_buffer_outdated {
        update_command_buffers(world, assets, state);
    }
}

pub fn render(state: &mut State) {
    let (image_i, suboptimal, acquire_future) = match swapchain::acquire_next_image(
        state.renderer.swapchain.as_ref().unwrap().clone(),
        None,
    )
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

    if let Some(image_fence) = &state.renderer.fences.as_ref().unwrap()[image_i as usize] {
        image_fence.wait(None).unwrap();
    }

    let previous_future =
        match state.renderer.fences.as_ref().unwrap()[state.renderer.previous_fence].clone() {
            None => {
                let mut now = sync::now(state.renderer.device.as_ref().unwrap().clone());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

    // Waiting for all fences to be able to write to buffers

    let future = previous_future
        .join(acquire_future)
        .then_execute(
            state.renderer.queue.as_ref().unwrap().clone(),
            state.renderer.command_buffers.as_ref().unwrap()[image_i as usize].clone(),
        )
        .unwrap()
        .then_swapchain_present(
            state.renderer.queue.as_ref().unwrap().clone(),
            SwapchainPresentInfo::swapchain_image_index(
                state.renderer.swapchain.as_ref().unwrap().clone(),
                image_i,
            ),
        )
        .then_signal_fence_and_flush();

    state.renderer.fences.as_mut().unwrap()[image_i as usize] =
        match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                state.renderer.recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };
    state.renderer.previous_fence = image_i as usize;
}

pub fn wait_for_idle(state: &mut State) {
    for fence in state.renderer.fences.as_mut().unwrap().iter_mut() {
        let _ = match fence.as_mut() {
            Some(val) => val.wait(None).unwrap(),
            _ => (),
        };
    }
}

pub fn init(state: &mut State) {
    state.renderer.library = Some(VulkanLibrary::new().expect("Vulkan library not found"));
    state.renderer.instance = Some(
        Instance::new(
            state.renderer.library.as_ref().unwrap().clone(),
            InstanceCreateInfo {
                enabled_extensions: Surface::required_extensions(&state.window.window_handle),
                ..Default::default()
            },
        )
        .unwrap(),
    );
    state.renderer.surface = Some(
        Surface::from_window(
            state.renderer.instance.as_ref().unwrap().clone(),
            state.window.window_handle.clone(),
        )
        .unwrap(),
    );
    select_physical_device(
        state,
        &DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        },
    );
    let (device, mut queues) = Device::new(
        state.renderer.physical_device.as_ref().unwrap().clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: *state.renderer.queue_family_index.as_ref().unwrap(),
                ..Default::default()
            }],
            enabled_extensions: DeviceExtensions {
                khr_swapchain: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    state.renderer.queue = Some(queues.next().unwrap());
    state.renderer.device = Some(device);
    state.renderer.memeory_allocator = Some(Arc::new(StandardMemoryAllocator::new_default(
        state.renderer.device.as_ref().unwrap().clone(),
    )));
    state.renderer.vp_buffer = Some(UpdatableBuffer::new(
        &state.renderer,
        BufferUsage::UNIFORM_BUFFER,
    ));
    get_swapchain(state);
    get_render_pass(state);
    get_framebuffers(state);
    state.renderer.viewport = Some(Viewport {
        offset: [0.0, 0.0],
        extent: state.window.window_handle.inner_size().into(),
        depth_range: 0.0..=1.0,
    });
    state.renderer.frames_in_flight = state.renderer.images.as_ref().unwrap().len();
    state.renderer.fences = Some(vec![None; state.renderer.frames_in_flight]);
}
impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            library: None,
            instance: None,
            surface: None,
            physical_device: None,
            queue_family_index: None,
            device: None,
            queue: None,
            memeory_allocator: None,
            render_pass: None,
            swapchain: None,
            images: None,
            framebuffers: None,
            viewport: None,
            command_buffers: None,
            window_resized: false,
            command_buffer_outdated: false,
            recreate_swapchain: false,
            frames_in_flight: 0,
            fences: None,
            previous_fence: 0,
            vp_data: VPData {
                view: Matrix4f::indentity(),
                projection: Matrix4f::indentity(),
            },
            vp_buffer: None,
            pipelines: HashMap::new(),
        }
    }
}
