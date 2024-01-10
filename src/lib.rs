use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet, DescriptorSet};
use vulkano::descriptor_set::allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
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
use vulkano::shader::spirv::bytes_to_words;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::future::FenceSignalFuture;
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub mod types;
use types::vectors::*;
use types::matrices::*;

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

#[repr(C)]
pub struct Mesh {
    pub mesh: Vec<VertexData>,
    pub vertex: String,
    pub fragment: String,
    pub buffer: Option<Subbuffer<[VertexData]>>,
}

#[derive(BufferContents, Clone, Copy)]
#[repr(C)]
pub struct VPData {
    pub view: Matrix4f,
    pub projection: Matrix4f,
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

pub fn read_file_to_words(path: &str) -> Vec<u32> {
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![0u8; file.metadata().unwrap().len() as usize];
    file.read(buffer.as_mut_slice()).unwrap();
    bytes_to_words(buffer.as_slice()).unwrap().to_vec()
}

fn load_shader_module(
    shaders: &HashMap<String, ShaderData>,
    device: &Arc<Device>,
    name: &str,
) -> Arc<ShaderModule> {
    unsafe {
        ShaderModule::new(
            device.clone(),
            ShaderModuleCreateInfo::new(shaders.get(name).unwrap().shader_code.as_slice()),
        )
        .unwrap()
    }
}

pub fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
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
        .expect("no device available")
}

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(), // set the format the same as the swapchain
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .unwrap()
}

fn get_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    mamory_allocator: Arc<StandardMemoryAllocator>,
) -> Vec<Arc<Framebuffer>> {
    let depth_buffer = ImageView::new_default(
        Image::new(
            mamory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
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
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state = VertexData::per_vertex()
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
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

static mut dbg: f32 = -3.0;

fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    queue: &Arc<Queue>,
    pipelines: &HashMap<(String, String), Arc<GraphicsPipeline>>,
    framebuffers: &[Arc<Framebuffer>],
    meshes: &Vec<Mesh>,
    vp_set: &Arc<PersistentDescriptorSet>,
    m_set: &Vec<Arc<PersistentDescriptorSet>>,
    m_buffer: &Vec<Subbuffer<Matrix4f>>,
    m_data: &Vec<Arc<Matrix4f>>
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into()), Some(1f32.into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .unwrap();

            for (i, mesh) in meshes.iter().enumerate() {

                let pipeline =  pipelines
                    .get(&(mesh.vertex.clone(), mesh.fragment.clone()))
                    .unwrap()
                    .clone();
                
                let model_set = PersistentDescriptorSet::new(
                    descriptor_set_allocator, 
                    pipeline.layout().set_layouts().get(1).unwrap().clone(), 
                    [WriteDescriptorSet::buffer(0, m_buffer.get(i).unwrap().clone())], 
                    []).unwrap();


                builder
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap()
                    .bind_descriptor_sets(PipelineBindPoint::Graphics, 
                                          pipeline.layout().clone(), 0, 
                                          (vp_set.clone(), model_set.clone()))
                    .unwrap()
                    .bind_vertex_buffers(0, mesh.buffer.clone().unwrap().clone())
                    .unwrap()
                    .draw(mesh.mesh.len() as u32, 1, 0, 0)
                    .unwrap();
            }

            builder.end_render_pass(Default::default()).unwrap();

            for (i, _) in meshes.iter().enumerate() {
                builder
                .update_buffer(m_buffer.get(i).unwrap().clone(), m_data.get(i).unwrap().clone())
                .unwrap();
            }

            builder.build().unwrap()
        })
        .collect()
}

pub fn run(mut meshes: Vec<Mesh>, shaders: HashMap<String, ShaderData>) {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let event_loop = EventLoop::new();

    let required_extensions = Surface::required_extensions(&event_loop);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) =
        select_physical_device(&instance, &surface, &device_extensions);

    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions, // new
            ..Default::default()
        },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap()
    };

    let standard_memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let render_pass = get_render_pass(device.clone(), swapchain.clone());
    let framebuffers = get_framebuffers(
        &images,
        render_pass.clone(),
        standard_memory_allocator.clone(),
    );

    for mesh in meshes.iter_mut() {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        mesh.buffer = Some(
            Buffer::from_iter(
                memory_allocator,
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                mesh.mesh.clone(),
            )
            .unwrap(),
        );
    }

    let mut model_data = vec![Arc::new(Matrix4f::translation(Vec3f([0.0, 0.0, -1.0]))); 2];

    let mut model_buffers: Vec<Subbuffer<Matrix4f>> = Vec::new();
    for i in (0..2).step_by(1) {
        println!("{}", i);
        model_buffers.push(
            Buffer::from_data(
                standard_memory_allocator.clone(), 
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                Matrix4f::translation(Vec3f([0.0, 0.0, 0.0]))
            ).unwrap()
        )
    }


    let vp_data = VPData {
        view: Matrix4f::look_at(
                  Vec3f([0.0, 0.0, 1.0]), 
                  Vec3f([0.0, 0.0, -1.0]), 
                  Vec3f([0.0, 1.0, 0.0])),
        projection: Matrix4f::perspective(1.57, 1.0, 0.01, 10.0)
    };
    println!("{:?}", vp_data.view);
    let vp_buffer = Buffer::from_data(
        standard_memory_allocator.clone(), 
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vp_data
    ).unwrap();

    let vertex_shaders: Vec<(&String, &ShaderData)> = shaders
        .iter()
        .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Vertex))
        .collect();

    let fragment_shaders: Vec<(&String, &ShaderData)> = shaders
        .iter()
        .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Fragment))
        .collect();

    let mut loaded_shaders: HashMap<String, Arc<ShaderModule>> = HashMap::new();

    for (shader, _) in shaders.iter() {
        loaded_shaders.insert(
            shader.to_string(),
            load_shader_module(&shaders, &device, shader),
        );
    }

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window.inner_size().into(),
        depth_range: 0.0..=1.0,
    };

    let mut pipelines: HashMap<(String, String), Arc<GraphicsPipeline>> = HashMap::new();
    for (name_vert, _) in vertex_shaders.iter() {
        for (name_frag, _) in fragment_shaders.iter() {
            pipelines.insert(
                (name_vert.to_string(), name_frag.to_string()),
                get_pipeline(
                    device.clone(),
                    loaded_shaders.get(*name_vert).unwrap().clone(),
                    loaded_shaders.get(*name_frag).unwrap().clone(),
                    render_pass.clone(),
                    viewport.clone(),
                ),
            );
        }
    }

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let vp_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        pipelines.get(&("simple".to_string(), "uv".to_string())).unwrap().layout().set_layouts().get(0).unwrap().clone(), 
        [WriteDescriptorSet::buffer(0, vp_buffer.clone())], 
        []).unwrap();
    
    let mut m_sets: Vec<Arc<PersistentDescriptorSet>> = Vec::new();
    for buff in model_buffers.iter() {
        m_sets.push(
            PersistentDescriptorSet::new(
                &descriptor_set_allocator,
                pipelines.get(&("simple".to_string(), "uv".to_string())).unwrap().layout().set_layouts().get(1).unwrap().clone(), 
                [WriteDescriptorSet::buffer(0, buff.clone())], 
                []
            ).unwrap()
        )
    }
    
    let mut command_buffers = get_command_buffers(
        &command_buffer_allocator,
        &descriptor_set_allocator,
        &queue,
        &pipelines,
        &framebuffers,
        &meshes,
        &vp_set,
        &m_sets,
        &model_buffers,
        &model_data
    );

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            window_resized = true;
        }
        Event::MainEventsCleared => {
            if window_resized || recreate_swapchain {
                recreate_swapchain = false;

                let new_dimensions = window.inner_size();

                let (new_swapchain, new_images) = swapchain
                    .recreate(SwapchainCreateInfo {
                        image_extent: new_dimensions.into(),
                        ..swapchain.create_info()
                    })
                    .expect("failed to recreate swapchain");

                swapchain = new_swapchain;
                let new_framebuffers = get_framebuffers(
                    &new_images,
                    render_pass.clone(),
                    standard_memory_allocator.clone(),
                );

                if window_resized {
                    window_resized = false;

                    viewport.extent = new_dimensions.into();
                    for (pipeline, val) in pipelines.iter_mut() {
                        *val = get_pipeline(
                            device.clone(),
                            loaded_shaders.get(&pipeline.0).unwrap().clone(),
                            loaded_shaders.get(&pipeline.1).unwrap().clone(),
                            render_pass.clone(),
                            viewport.clone(),
                        )
                    }
                    command_buffers = get_command_buffers(
                        &command_buffer_allocator,
                        &descriptor_set_allocator,
                        &queue,
                        &pipelines,
                        &new_framebuffers,
                        &meshes,
                        &vp_set,
                        &m_sets,
                        &model_buffers,
                        &model_data
                    );
                }
            }

            unsafe {
            *model_data.get_mut(0).unwrap() = Arc::new(Matrix4f::translation(Vec3f([0.0, 0.0, dbg.clone()])));
            *model_data.get_mut(1).unwrap() = Arc::new(Matrix4f::translation(Vec3f([0.0, 0.0, dbg.clone()])));
            dbg += 0.001;
            dbg %= 5.0;
            println!("{:?}", dbg);
            }
            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            // wait for the fence related to this image to finish (normally this would be the oldest fence)
            if let Some(image_fence) = &fences[image_i as usize] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i as usize].clone() {
                // Create a NowFuture
                None => {
                    let mut now = sync::now(device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
                )
                .then_signal_fence_and_flush();

            fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                Ok(value) => Some(Arc::new(value)),
                Err(VulkanError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("failed to flush future: {e}");
                    None
                }
            };
            previous_fence_i = image_i;
        }
        _ => (),
    });
}
