use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, CopyBufferInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
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
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::spirv::{bytes_to_words, ImageFormat};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use winit::window::WindowBuilder;

use crate::types::vectors::*;
use crate::types::matrices::*;
use crate::types::buffers::*;

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct VertexData {
    #[format(R32G32B32_SFLOAT)]
    pub position: Vec3f,
    #[format(R32G32_SFLOAT)]
    pub uv: Vec2f,
    #[format(R32G32B32_SFLOAT)]
    pub normal: Vec3f,
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct VPData {
    pub view: Matrix4f,
    pub projection: Matrix4f,
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ModelData {
    pub translation: Matrix4f
}

pub struct Shader {
    pub shader: Arc<ShaderModule>,
}

pub enum ShaderType {
    Fragment,
    Vertex,
}

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
        ShaderManager { library: HashMap::new(), pipelines: HashMap::new() }
    }
}

pub struct Mesh {
    pub mesh: Vec<VertexData>,
    pub vertex: String,
    pub fragment: String,
    pub buffer: Option<Subbuffer<[VertexData]>>,
}

pub struct MeshManager {
    pub list: Vec<Mesh>
}

impl MeshManager {
    pub fn new() -> MeshManager {
        MeshManager { list: Vec::new() }
    }
}

pub struct Window {
    pub window_handle: Arc<winit::window::Window>,
}

impl Window {
    pub fn new(event_loop: &EventLoop) -> Window {
        Window { window_handle: Arc::new(WindowBuilder::new().build(&event_loop.event_loop).unwrap()) }
    }
}

pub struct EventLoop {
    pub event_loop: winit::event_loop::EventLoop<()>,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop { event_loop: winit::event_loop::EventLoop::new() }
    }
}

pub struct SimpleRenderer<T: GpuFuture> {
    library: Option<Arc<VulkanLibrary>>,
    instance: Option<Arc<Instance>>,
    surface: Option<Arc<Surface>>,
    physical_device: Option<Arc<PhysicalDevice>>,
    queue_family_index: Option<u32>,
    pub device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    pub render_pass: Option<Arc<RenderPass>>,
    swapchain: Option<Arc<Swapchain>>,
    images: Option<Vec<Arc<Image>>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
    pub viewport: Option<Viewport>,
    command_buffers: Option<Vec<Arc<PrimaryAutoCommandBuffer>>>,
    pub window_resized: bool,
    pub recreate_swapchain: bool,
    pub frames_in_flight: usize,
    pub fences: Option<Vec<Option<Arc<FenceSignalFuture<T>>>>>,
    pub previous_fence: usize,
}

impl <T> SimpleRenderer<T>
where
    T: GpuFuture
{
    fn select_physical_device(&mut self, device_extensions: &DeviceExtensions) {
        let (physical_device, queue_family_index) = self.instance.clone().unwrap()
            .enumerate_physical_devices().expect("failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &self.surface.clone().unwrap()).unwrap_or(false)
                    })
                .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            }).expect("no device available");

        self.physical_device = Some(physical_device);
        self.queue_family_index = Some(queue_family_index);
    }

    fn get_render_pass(&mut self) {
        self.render_pass = Some(vulkano::single_pass_renderpass!(
            self.device.clone().unwrap().clone(),
            attachments: {
                inter: {
                    format: self.swapchain.clone().unwrap().image_format(), // set the format the same as the swapchain
                    samples: 8,
                    load_op: Clear,
                    store_op: Store,
                },
                color: {
                    format: self.swapchain.clone().unwrap().image_format(), // set the format the same as the swapchain
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
        ).unwrap())
    }

    fn get_framebuffers(&mut self) {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(self.device.clone().unwrap().clone()));

        let depth_buffer = ImageView::new_default(
            Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: self.images.clone().unwrap()[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    samples: SampleCount::Sample8,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
                )
            .unwrap(),
            )
            .unwrap();


        self.framebuffers = Some(self.images.clone().unwrap()
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
                        AllocationCreateInfo::default()
                    ).unwrap()
                ).unwrap();

                Framebuffer::new(
                    self.render_pass.clone().unwrap().clone(),
                    FramebufferCreateInfo {
                        attachments: vec![inter, view, depth_buffer.clone()],
                        ..Default::default()
                    }
                ).unwrap()
            })
            .collect::<Vec<_>>())
    }

    pub fn get_pipeline(&mut self, vs: &Shader, fs: &Shader) -> Arc<GraphicsPipeline> {
        let vs = vs.shader.entry_point("main").unwrap();
        let fs = fs.shader.entry_point("main").unwrap();

        let vertex_input_state = VertexData::per_vertex()
            .definition(&vs.info().input_interface).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.clone().unwrap().clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(self.device.clone().unwrap().clone()).unwrap()).unwrap();

        let subpass = Subpass::from(self.render_pass.clone().unwrap().clone(), 0).unwrap();

        GraphicsPipeline::new(
            self.device.clone().unwrap().clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [self.viewport.clone().unwrap().clone()].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState { rasterization_samples: SampleCount::Sample8, ..Default::default() }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        ).unwrap()
    }

    pub fn update_command_buffers(&mut self, meshes: &MeshManager, shaders: &ShaderManager, m_buffers: &Vec<UpdatableBuffer<ModelData>>, vp_buffer: &UpdatableBuffer<VPData>) {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(self.device.clone().unwrap().clone(), Default::default());
        let command_buffer_allocator = StandardCommandBufferAllocator::new(self.device.clone().unwrap().clone(), Default::default());

        self.command_buffers = Some(self.framebuffers.clone().unwrap()
            .iter()
            .map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    self.queue.clone().unwrap().queue_family_index(),
                    CommandBufferUsage::MultipleSubmit,
                ).unwrap();

                for buff in m_buffers.iter() {
                    builder
                        .copy_buffer(
                            CopyBufferInfo::buffers(buff.staging_buffer.clone(),
                            buff.main_buffer.clone())
                        ).unwrap();
                }

                builder
                    .copy_buffer(
                        CopyBufferInfo::buffers(vp_buffer.staging_buffer.clone(), 
                                                vp_buffer.main_buffer.clone())).unwrap()
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into()), Some([0.1, 0.1, 0.1, 1.0].into()), Some(1f32.into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    ).unwrap();

                for (i, mesh) in meshes.list.iter().enumerate() {
                    let pipeline = shaders.pipelines
                        .get(&(mesh.vertex.clone(), mesh.fragment.clone())).unwrap().clone();

                    let vp_set = PersistentDescriptorSet::new(
                        &descriptor_set_allocator, 
                        pipeline.layout().set_layouts().get(0).unwrap().clone(), 
                        [WriteDescriptorSet::buffer(0, vp_buffer.main_buffer.clone())], 
                        []).unwrap();

                    let m_set = PersistentDescriptorSet::new(
                        &descriptor_set_allocator, 
                        pipeline.layout().set_layouts().get(1).unwrap().clone(), 
                        [WriteDescriptorSet::buffer(0, m_buffers.get(i).unwrap().main_buffer.clone())], 
                        []).unwrap();


                    builder
                        .bind_pipeline_graphics(pipeline.clone()).unwrap()
                        .bind_descriptor_sets(PipelineBindPoint::Graphics, 
                                              pipeline.layout().clone(), 0, 
                                              (vp_set.clone(), m_set.clone())).unwrap()
                        .bind_vertex_buffers(0, mesh.buffer.clone().unwrap().clone()).unwrap()
                        .draw(mesh.mesh.len() as u32, 1, 0, 0).unwrap();
                }

                builder.end_render_pass(Default::default()).unwrap();
                builder.build().unwrap()
        })
        .collect())
    }

    pub fn new() -> SimpleRenderer<T> {
        SimpleRenderer { 
            library: None, 
            instance: None, 
            surface: None,
            physical_device: None, 
            queue_family_index: None, 
            device: None, 
            queue: None, 
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
            previous_fence: 0
        }
    }

    pub fn init(&mut self, event_loop: &EventLoop, window: &Window) {
        self.library = Some(VulkanLibrary::new().expect("Vulkan library not found"));
        self.instance = Some(Instance::new(
            self.library.clone().unwrap(),
            InstanceCreateInfo {
                enabled_extensions: Surface::required_extensions(&event_loop.event_loop),
                ..Default::default()
            }
        ).unwrap());
        self.surface = Some(Surface::from_window(self.instance.clone().unwrap(), window.window_handle.clone()).unwrap());
        self.select_physical_device(&DeviceExtensions { khr_swapchain: true, ..Default::default() });
        let (device, mut queues) = Device::new(
            self.physical_device.clone().unwrap(), 
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: self.queue_family_index.clone().unwrap(),
                    ..Default::default()
                }],
                enabled_extensions: DeviceExtensions { khr_swapchain: true, ..Default::default() },
                ..Default::default()
            }
        ).unwrap();
        self.queue = Some(queues.next().unwrap());
        self.device = Some(device);
        let (swapchain, images) = {
            let caps = self.physical_device.clone().unwrap()
                .surface_capabilities(&self.surface.clone().unwrap(), Default::default())
                .expect("failed to get surface capabilities");

            let dimensions = window.window_handle.inner_size();
            let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
            let image_format = self.physical_device.clone().unwrap().surface_formats(&self.surface.clone().unwrap(), Default::default()).unwrap()[0].0;

            Swapchain::new(
                self.device.clone().unwrap().clone(),
                self.surface.clone().unwrap(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format,
                    image_extent: dimensions.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                    composite_alpha,
                    ..Default::default()
                },
            ).unwrap()
        };
        self.swapchain = Some(swapchain);
        self.images = Some(images);
        self.get_render_pass();
        self.get_framebuffers();
        self.viewport = Some(Viewport {
            offset: [0.0, 0.0],
            extent: window.window_handle.inner_size().into(),
            depth_range: 0.0..=1.0,
        });
        self.frames_in_flight = self.images.clone().unwrap().len();
        self.fences = Some(vec![None; self.frames_in_flight]);
    }
}
