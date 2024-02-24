use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, CopyBufferInfo,
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

use crate::ecs::{World, System};
use crate::types::buffers::*;
use crate::types::matrices::*;
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
    fn on_start(&self, _world: &World, _renderer: &mut Renderer) {}

    fn on_update(&self, world: &World, renderer: &mut Renderer) {
        let mut camera = world.borrow_component_vec_mut::<Camera>().unwrap();
        let mut transform = world.borrow_component_vec_mut::<Transform>().unwrap();
        let zip = camera.iter_mut().zip(transform.iter_mut());
        let mut iter = zip.filter_map(|(camera, transform)| { Some((camera.as_mut()?, transform.as_mut()?)) });
        let (_, transform_data) = iter.next().unwrap();
        let cam_rot = Matrix4f::rotation_yxz(transform_data.rotation);
        renderer.vp_data.view = Matrix4f::look_at(
            transform_data.position.to_vec3f(), 
            cam_rot.vec_mul(Vec3f::new([1.0, 0.0, 0.0])),
            cam_rot.vec_mul(Vec3f::new([0.0, 1.0, 0.0])),
            );
        renderer.vp_buffer.as_mut().unwrap().write(renderer.vp_data);
    }
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ModelData {
    pub translation: Matrix4f,
}

pub struct Shader {
    pub shader: Arc<ShaderModule>,
}

fn load_shader_module(
    shaders: &HashMap<String, ShaderData>,
    device: &Arc<Device>,
    name: &str,
) -> Shader {
    unsafe {
        Shader {
            shader: ShaderModule::new(
                device.clone(),
                ShaderModuleCreateInfo::new(shaders.get(name).unwrap().shader_code.as_slice()),
            )
            .unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum ShaderType {
    Fragment,
    Vertex,
}

#[derive(Debug)]
pub struct ShaderData {
    pub shader_code: Vec<u32>,
    pub shader_type: ShaderType,
}

pub struct ShaderManager {
    pub library: HashMap<String, Shader>,
    pub pipelines: HashMap<(String, String), Arc<GraphicsPipeline>>,
}

impl ShaderManager {
    pub fn new() -> ShaderManager {
        ShaderManager {
            library: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    pub fn load(&mut self, renderer: &mut Renderer) {
        let mut shaders: HashMap<String, ShaderData> = HashMap::new();
        let mut shaders_file = fs::File::open("./shaders/bin/.shaders").unwrap();
        let mut shaders_db = String::new();
        shaders_file.read_to_string(&mut shaders_db).unwrap();
        for shader in shaders_db.lines() {
            let (name, shader_type) = shader.split_once(" ").unwrap();
            let shader_type = if shader_type == "frag" {
                ShaderType::Fragment
            } else {
                ShaderType::Vertex
            };
            let mut shader_path = "./shaders/bin/".to_owned();
            shader_path.push_str(name);
            shader_path.push_str(".spv");
            shaders.insert(
                name.to_string(),
                ShaderData {
                    shader_code: read_file_to_words(&shader_path),
                    shader_type,
                },
                );
        }

        let vertex_shaders: Vec<(&String, &ShaderData)> = shaders
            .iter()
            .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Vertex))
            .collect();

        let fragment_shaders: Vec<(&String, &ShaderData)> = shaders
            .iter()
            .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Fragment))
            .collect();

        for (shader, _) in shaders.iter() {
            self.library.insert(
                shader.to_string(),
                load_shader_module(&shaders, &renderer.device.clone().unwrap(), shader),
            );
        }

        for (name_vert, _) in vertex_shaders.iter() {
            for (name_frag, _) in fragment_shaders.iter() {
                self.pipelines.insert(
                    (name_vert.to_string(), name_frag.to_string()),
                    renderer.get_pipeline(
                        self.library.get(*name_vert).unwrap(),
                        self.library.get(*name_frag).unwrap(),
                    ),
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
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

pub struct MeshUpdater {}

impl System for MeshUpdater {
    fn on_start(&self, world: &World, renderer: &mut Renderer) {
        for mesh in world
            .borrow_component_vec_mut::<Mesh>()
            .unwrap()
            .iter_mut()
            .filter(|x| x.is_some())
        {
                mesh.as_mut().unwrap().load(renderer);
        }
    }

    fn on_update(&self, _world: &World, _renderer: &mut Renderer) {}
}

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
}

impl Renderer {
    fn select_physical_device(&mut self, device_extensions: &DeviceExtensions) {
        let (physical_device, queue_family_index) = self
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
                            && p.surface_support(i as u32, &self.surface.clone().unwrap())
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

        self.physical_device = Some(physical_device);
        self.queue_family_index = Some(queue_family_index);
    }

    fn get_render_pass(&mut self) {
        self.render_pass = Some(
            vulkano::single_pass_renderpass!(
                self.device.as_ref().unwrap().clone(),
                attachments: {
                    inter: {
                        format: self.swapchain.as_ref().unwrap().image_format(), // set the format the same as the swapchain
                        samples: 8,
                        load_op: Clear,
                        store_op: Store,
                    },
                    color: {
                        format: self.swapchain.as_ref().unwrap().image_format(), // set the format the same as the swapchain
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

    fn get_framebuffers(&mut self) {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(
            self.device.as_ref().unwrap().clone(),
        ));

        let depth_buffer = ImageView::new_default(
            Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: self.images.as_ref().unwrap()[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    samples: SampleCount::Sample8,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        self.framebuffers = Some(
            self.images
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
                        self.render_pass.as_ref().unwrap().clone(),
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

    pub fn get_pipeline(&mut self, vs: &Shader, fs: &Shader) -> Arc<GraphicsPipeline> {
        let vs = vs.shader.entry_point("main").unwrap();
        let fs = fs.shader.entry_point("main").unwrap();

        let vertex_input_state = VertexData::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.as_ref().unwrap().clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.as_ref().unwrap().clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap();

        GraphicsPipeline::new(
            self.device.as_ref().unwrap().clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [self.viewport.as_ref().unwrap().clone()]
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

    pub fn update_command_buffers(
        &mut self,
        world: &mut World,
        shaders: &ShaderManager,
    ) {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            self.device.as_ref().unwrap().clone(),
            Default::default(),
        );
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            self.device.as_ref().unwrap().clone(),
            Default::default(),
        );

        self.command_buffers = Some(
            self.framebuffers
                .as_ref()
                .unwrap()
                .iter()
                .map(|framebuffer| {
                    let mut meshes = world.borrow_component_vec_mut::<Mesh>().unwrap();
                    let mut transforms = world.borrow_component_vec_mut::<Transform>().unwrap();

                    let mut builder = AutoCommandBufferBuilder::primary(
                        &command_buffer_allocator,
                        self.queue.as_ref().unwrap().queue_family_index(),
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
                            self.vp_buffer.as_ref().unwrap().staging_buffer.clone(),
                            self.vp_buffer.as_ref().unwrap().main_buffer.clone(),
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
                    let iter = zip.filter_map(|(mesh, transform)| {
                        Some((mesh.as_mut()?, transform.as_mut()?))
                    });
                    // println!("{}", iter.count());

                    for (mesh, transform) in iter {
                        let pipeline = shaders
                            .pipelines
                            .get(&(mesh.vertex.clone(), mesh.fragment.clone()))
                            .unwrap()
                            .clone();

                        let vp_set = PersistentDescriptorSet::new(
                            &descriptor_set_allocator,
                            pipeline.layout().set_layouts().get(0).unwrap().clone(),
                            [WriteDescriptorSet::buffer(
                                0,
                                self.vp_buffer.as_ref().unwrap().main_buffer.clone(),
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

    fn get_swapchain(&mut self, window: &Window) {
        let (swapchain, images) = {
            let caps = self
                .physical_device
                .as_ref()
                .unwrap()
                .surface_capabilities(&self.surface.as_ref().unwrap(), Default::default())
                .expect("failed to get surface capabilities");

            let dimensions = window.window_handle.inner_size();
            let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
            let image_format = self
                .physical_device
                .as_ref()
                .unwrap()
                .surface_formats(&self.surface.as_ref().unwrap(), Default::default())
                .unwrap()[0]
                .0;

            Swapchain::new(
                self.device.as_ref().unwrap().clone(),
                self.surface.as_ref().unwrap().clone(),
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
        self.swapchain = Some(swapchain);
        self.images = Some(images);
    }

    pub fn handle_possible_resize(
        &mut self,
        window: &Window,
        world: &mut World,
        shaders: &mut ShaderManager,
    ) {
        if self.window_resized || self.recreate_swapchain {
            self.recreate_swapchain = false;
            self.window_resized = false;

            let new_dimensions = window.window_handle.inner_size();

            let (new_swapchain, new_images) = self
                .swapchain
                .as_ref()
                .unwrap()
                .recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.as_ref().unwrap().create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = Some(new_swapchain);
            self.images = Some(new_images);
            self.get_framebuffers();
            
            let camera = world.borrow_component_vec_mut::<Camera>().unwrap();
            let transform = world.borrow_component_vec_mut::<Transform>().unwrap();
            let zip = camera.iter().zip(transform.iter());
            let mut iter = zip.filter_map(|(camera, transform)| { Some((camera.as_ref()?, transform.as_ref()?)) });
            let (camera_data, _) = iter.next().unwrap();
            self.vp_data.projection = Matrix4f::perspective(
                camera_data.vfov.to_radians(),
                (new_dimensions.width as f32) / (new_dimensions.height as f32),
                camera_data.near,
                camera_data.far,
            );

            self.viewport.as_mut().unwrap().extent = new_dimensions.into();
            for (pipeline, val) in shaders.pipelines.iter_mut() {
                *val = self.get_pipeline(
                    shaders.library.get(&pipeline.0).unwrap(),
                    shaders.library.get(&pipeline.1).unwrap(),
                )
            }

            drop(camera);
            drop(transform);
            self.update_command_buffers(world, shaders);
        }
    }

    pub fn render(&mut self) {
        let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.as_ref().unwrap().clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        if let Some(image_fence) = &self.fences.as_ref().unwrap()[image_i as usize] {
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences.as_ref().unwrap()[self.previous_fence].clone() {
            None => {
                let mut now = sync::now(self.device.as_ref().unwrap().clone());
                now.cleanup_finished();
                now.boxed()
            }
            Some(fence) => fence.boxed(),
        };

        // Waiting for all fences to be able to write to buffers

        let future = previous_future
            .join(acquire_future)
            .then_execute(
                self.queue.as_ref().unwrap().clone(),
                self.command_buffers.as_ref().unwrap()[image_i as usize].clone(),
            )
            .unwrap()
            .then_swapchain_present(
                self.queue.as_ref().unwrap().clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.as_ref().unwrap().clone(),
                    image_i,
                ),
            )
            .then_signal_fence_and_flush();

        self.fences.as_mut().unwrap()[image_i as usize] = match future.map_err(Validated::unwrap) {
            Ok(value) => Some(Arc::new(value)),
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                None
            }
        };
        self.previous_fence = image_i as usize;
    }

    pub fn wait_for_idle(&mut self) {
        for fence in self.fences.as_mut().unwrap().iter_mut() {
            let _ = match fence.as_mut() {
                Some(val) => val.wait(None).unwrap(),
                _ => (),
            };
        }
    }

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
            recreate_swapchain: false,
            frames_in_flight: 0,
            fences: None,
            previous_fence: 0,
            vp_data: VPData { view: Matrix4f::indentity(), projection: Matrix4f::indentity() },
            vp_buffer: None
        }
    }

    pub fn init(&mut self, event_loop: &EventLoop, window: &Window) {
        self.library = Some(VulkanLibrary::new().expect("Vulkan library not found"));
        self.instance = Some(
            Instance::new(
                self.library.as_ref().unwrap().clone(),
                InstanceCreateInfo {
                    enabled_extensions: Surface::required_extensions(&event_loop.event_loop),
                    ..Default::default()
                },
            )
            .unwrap(),
        );
        self.surface = Some(
            Surface::from_window(
                self.instance.as_ref().unwrap().clone(),
                window.window_handle.clone(),
            )
            .unwrap(),
        );
        self.select_physical_device(&DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        });
        let (device, mut queues) = Device::new(
            self.physical_device.as_ref().unwrap().clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: *self.queue_family_index.as_ref().unwrap(),
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
        self.queue = Some(queues.next().unwrap());
        self.device = Some(device);
        self.memeory_allocator = Some(Arc::new(StandardMemoryAllocator::new_default(
            self.device.as_ref().unwrap().clone(),
        )));
        self.vp_buffer = Some(UpdatableBuffer::new(&self, BufferUsage::UNIFORM_BUFFER));
        self.get_swapchain(window);
        self.get_render_pass();
        self.get_framebuffers();
        self.viewport = Some(Viewport {
            offset: [0.0, 0.0],
            extent: window.window_handle.inner_size().into(),
            depth_range: 0.0..=1.0,
        });
        self.frames_in_flight = self.images.as_ref().unwrap().len();
        self.fences = Some(vec![None; self.frames_in_flight]);
    }
}
