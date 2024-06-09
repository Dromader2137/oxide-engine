use std::collections::HashMap;
use std::sync::Arc;
use std::usize;

use bytemuck::{Pod, Zeroable};

use log::{debug, error};

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecFuture, CommandBufferUsage, DrawIndirectCommand,
    PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{
    AttachmentBlend, ColorBlendAttachmentState, ColorBlendState, ColorComponents,
};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::swapchain::{
    self, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    SwapchainPresentInfo,
};
use vulkano::sync::future::{FenceSignalFuture, JoinFuture};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary};

use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;

use crate::asset_library::AssetLibrary;
use crate::ecs::{System, World};
use crate::state::State;
use crate::types::camera::Camera;
use crate::types::material::Attachment;
use crate::types::matrices::*;
use crate::types::mesh::DynamicMesh;
use crate::types::model::ModelComponent;
use crate::types::shader::Shader;
use crate::types::transform::{ModelData, Transform};
use crate::types::vectors::*;

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct VertexData {
    pub position: Vec3f,
    pub uv: Vec2f,
    pub normal: Vec3f,
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

#[derive(Clone)]
pub struct MeshBuffers {
    pub vertex_ptr: Option<Subbuffer<[u64]>>,
    pub index_ptr: Option<Subbuffer<[u64]>>,
    pub model: Option<Subbuffer<[ModelData]>>,
    pub indirect_draw: Option<Subbuffer<[DrawIndirectCommand]>>,
}

impl MeshBuffers {
    pub fn new() -> MeshBuffers {
        MeshBuffers {
            vertex_ptr: None,
            index_ptr: None,
            indirect_draw: None,
            model: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Renderer {
    library: Arc<VulkanLibrary>,
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,

    surface: Arc<Surface>,

    queue_family_index: u32,
    transfer_family_index: Option<u32>,

    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub transfer_queue: Option<Arc<Queue>>,

    pub memeory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

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
    pub previous_fence: usize,

    pub pipelines: HashMap<(String, String), Arc<GraphicsPipeline>>,
    pub dynamic_mesh_data: HashMap<String, MeshBuffers>,
}

fn select_physical_device(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device_extensions: &DeviceExtensions,
    features: &Features,
) -> (Arc<PhysicalDevice>, u32, Option<u32>) {
    instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter(|p| p.supported_features().contains(features))
        .filter_map(|p| {
            let gq = p
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface.clone())
                            .unwrap_or(false)
                })
                .map(|q| q as u32);
            let tq = p
                .queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::TRANSFER)
                        && i as u32 != gq.expect("No graphics queue")
                })
                .map(|q| q as u32);

            debug!("Selected queues main:{:?}, transfer:{:?}", gq, tq);

            if gq.is_some() {
                Some((p, gq.unwrap(), tq))
            } else {
                None
            }
        })
        .min_by_key(|(p, _, _)| match p.properties().device_type {
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
        device.clone(),
        attachments: {
            inter: {
                format: swapchain.image_format(),
                samples: 8,
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

pub fn get_pipeline(state: &State, vs: &Shader, fs: &Shader) -> Arc<GraphicsPipeline> {
    let vs = vs.module.as_ref().unwrap().entry_point("main").unwrap();
    let fs = fs.module.as_ref().unwrap().entry_point("main").unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs),
    ];

    let layout = PipelineLayout::new(
        state.renderer.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(state.renderer.device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(state.renderer.render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        state.renderer.device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(VertexInputState::new()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [state.renderer.viewport.clone()].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
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

fn prepare_meshes(world: &World, assets: &AssetLibrary, state: &mut State, material: &String) {
    let mut dynamic_query = world.entities.query::<(&mut DynamicMesh, &Transform)>();
    let mut filtered_by_material: Vec<_> = dynamic_query
        .iter()
        .filter(|x| x.1 .0.material == *material)
        .collect();
    let pmb = match state.renderer.dynamic_mesh_data.get_mut(material) {
        Some(val) => val,
        None => {
            state
                .renderer
                .dynamic_mesh_data
                .insert(material.clone(), MeshBuffers::new());
            state.renderer.dynamic_mesh_data.get_mut(material).unwrap()
        }
    };

    let camera_pos = state.renderer.vp_pos;
    filtered_by_material.sort_by(|a, b| {
        (a.1 .1.position - camera_pos)
            .length_sqr()
            .total_cmp(&(b.1 .1.position - camera_pos).length_sqr())
    });

    let mut counter: u32 = 0;
    let mut vertex_ptr = Vec::new();
    let mut index_ptr = Vec::new();
    let mut model = Vec::new();
    let mut indirect = Vec::new();

    for (_, (mesh, transform)) in filtered_by_material {
        if mesh.mesh.is_none() {
            continue;
        }

        let mesh_data = state.meshes.get(mesh.mesh.as_ref().unwrap()).unwrap();
        if mesh_data.vertices.len() == 0 {
            continue;
        }

        vertex_ptr.push(mesh_data.vertex_buffer.device_address().unwrap().get());
        index_ptr.push(mesh_data.index_buffer.device_address().unwrap().get());

        model.push(ModelData {
            translation: Matrix4f::translation((transform.position - camera_pos).to_vec3f()),
            rotation: Matrix4f::rotation_yxz(transform.rotation),
            scale: Matrix4f::scale(transform.scale)
        });
        indirect.push(DrawIndirectCommand {
            instance_count: 1,
            first_instance: counter,
            vertex_count: mesh_data.indices.len() as u32,
            first_vertex: 0,
        });

        counter += 1;
    }

    for (_, (model_comp, transform)) in world
        .entities
        .query::<(&ModelComponent, &Transform)>()
        .iter()
    {
        for (mesh_name, _) in assets
            .models
            .iter()
            .find(|x| x.name == model_comp.model_name)
            .unwrap()
            .meshes_and_materials
            .iter()
            .filter(|x| x.1 == *material)
        {
            let mesh_data = state.meshes.get(mesh_name).unwrap();
            if mesh_data.vertices.len() == 0 {
                continue;
            }

            vertex_ptr.push(mesh_data.vertex_buffer.device_address().unwrap().get());
            index_ptr.push(mesh_data.index_buffer.device_address().unwrap().get());

            model.push(ModelData {
                translation: Matrix4f::translation((transform.position - camera_pos).to_vec3f()),
                rotation: Matrix4f::rotation_yxz(transform.rotation),
                scale: Matrix4f::scale(transform.scale)
            });
            indirect.push(DrawIndirectCommand {
                instance_count: 1,
                first_instance: counter,
                vertex_count: mesh_data.indices.len() as u32,
                first_vertex: 0,
            });

            counter += 1;
        }
    }

    pmb.model = if model.len() > 0 {
        Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                model,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.vertex_ptr = if vertex_ptr.len() > 0 {
        Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertex_ptr,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.index_ptr = if index_ptr.len() > 0 {
        Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                index_ptr,
            )
            .unwrap(),
        )
    } else {
        None
    };
    pmb.indirect_draw = if indirect.len() > 0 {
        Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDIRECT_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indirect,
            )
            .unwrap(),
        )
    } else {
        None
    };
}

fn get_command_buffers(
    _world: &World,
    assets: &AssetLibrary,
    state: &mut State,
    image_id: usize,
) -> Arc<PrimaryAutoCommandBuffer> {
    let framebuffer = state.renderer.framebuffers.get(image_id).unwrap();
    let mut builder = AutoCommandBufferBuilder::primary(
        state.renderer.command_buffer_allocator.as_ref(),
        state.renderer.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.0, 0.0, 0.0, 1.0].into()),
                    Some([0.0, 0.0, 0.0, 1.0].into()),
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

    for (key, entry) in state.renderer.dynamic_mesh_data.iter() {
        if entry.vertex_ptr.is_none() || entry.model.is_none() || entry.indirect_draw.is_none() {
            continue;
        }

        let material = assets.materials.iter().find(|x| x.name == *key).unwrap();
        let pipeline = state
            .renderer
            .pipelines
            .get(&(
                material.vertex_shader.clone(),
                material.fragment_shader.clone(),
            ))
            .unwrap()
            .clone();

        builder.bind_pipeline_graphics(pipeline.clone()).unwrap();

        let vp_set = PersistentDescriptorSet::new(
            state.renderer.descriptor_set_allocator.as_ref(),
            pipeline.layout().set_layouts().first().unwrap().clone(),
            [WriteDescriptorSet::buffer(
                0,
                state.renderer.vp_buffers.get(image_id).unwrap().clone(),
            )],
            [],
        )
        .unwrap();

        let m_set = PersistentDescriptorSet::new(
            state.renderer.descriptor_set_allocator.as_ref(),
            pipeline.layout().set_layouts().get(1).unwrap().clone(),
            [WriteDescriptorSet::buffer(
                0,
                entry.model.as_ref().unwrap().clone(),
            )],
            [],
        )
        .unwrap();

        let vertex_set = PersistentDescriptorSet::new(
            state.renderer.descriptor_set_allocator.as_ref(),
            pipeline.layout().set_layouts().get(2).unwrap().clone(),
            [
                WriteDescriptorSet::buffer(0, entry.vertex_ptr.as_ref().unwrap().clone()),
                WriteDescriptorSet::buffer(1, entry.index_ptr.as_ref().unwrap().clone()),
            ],
            [],
        )
        .unwrap();

        let attachment_set = if !material.attachments.is_empty() {
            Some({
                PersistentDescriptorSet::new(
                    state.renderer.descriptor_set_allocator.as_ref(),
                    pipeline.layout().set_layouts().get(3).unwrap().clone(),
                    material
                        .attachments
                        .iter()
                        .enumerate()
                        .map(|(id, attachment)| match attachment {
                            Attachment::Texture(name) => {
                                let tex = assets.textures.iter().find(|x| x.name == *name).unwrap();
                                WriteDescriptorSet::image_view_sampler(
                                    id as u32,
                                    tex.image_view.as_ref().unwrap().clone(),
                                    tex.sampler.as_ref().unwrap().clone(),
                                )
                            }
                        })
                        .collect::<Vec<_>>(),
                    [],
                )
                .unwrap()
            })
        } else {
            None
        };

        match attachment_set {
            Some(set) => {
                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        (vp_set, m_set, vertex_set, set),
                    )
                    .unwrap();
            }
            None => {
                builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        (vp_set, m_set, vertex_set),
                    )
                    .unwrap();
            }
        }

        builder
            .draw_indirect(entry.indirect_draw.as_ref().unwrap().clone())
            .unwrap();
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
    let iter: Vec<(String, String)> = state.renderer.pipelines.keys().cloned().collect();
    for pipeline in iter.iter() {
        state.renderer.pipelines.insert(
            pipeline.clone(),
            get_pipeline(
                state,
                assets
                    .shaders
                    .iter()
                    .find(|x| x.name == pipeline.0)
                    .unwrap(),
                assets
                    .shaders
                    .iter()
                    .find(|x| x.name == pipeline.1)
                    .unwrap(),
            ),
        );
    }
}

fn recalculate_projection(world: &World, state: &mut State, new_dimensions: PhysicalSize<u32>) {
    let mut camera = world.entities.query::<&Camera>();
    let camera_data = camera.iter().next().expect("Camera not found").1;
    state.renderer.vp_data.projection = Matrix4f::perspective(
        camera_data.vfov.to_radians(),
        (new_dimensions.width as f32) / (new_dimensions.height as f32),
        camera_data.near,
        camera_data.far,
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
            state.renderer.device.clone(),
            &state.renderer.images,
            state.renderer.render_pass.clone(),
        );

        state.renderer.viewport.extent = new_dimensions.into();

        recalculate_projection(world, state, new_dimensions);
        recreate_pipelines(assets, state);
    }
}

#[allow(clippy::arc_with_non_send_sync)]
fn render(world: &World, assets: &AssetLibrary, state: &mut State) {
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

    for mat in assets.materials.iter() {
        prepare_meshes(world, assets, state, &mat.name);
    }

    let command_buffer = get_command_buffers(world, assets, state, image_i as usize);
    if let Some(image_fence) = &state.renderer.fences[image_i as usize] {
        image_fence.wait(None).unwrap();
    }

    let previous_future = match state.renderer.fences[state.renderer.previous_fence].clone() {
        None => {
            let mut now = sync::now(state.renderer.device.clone());
            now.cleanup_finished();
            now.boxed()
        }
        Some(fence) => fence.boxed(),
    };

    {
        let mut contents = state
            .renderer
            .vp_buffers
            .get(image_i as usize)
            .unwrap()
            .write()
            .unwrap();
        *contents = state.renderer.vp_data;
    }

    let future = previous_future
        .join(acquire_future)
        .then_execute(state.renderer.queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(
            state.renderer.queue.clone(),
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
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let features = Features {
            shader_draw_parameters: true,
            multi_draw_indirect: true,
            buffer_device_address: true,
            runtime_descriptor_array: true,
            ..Features::empty()
        };
        let extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_shader_draw_parameters: true,
            khr_buffer_device_address: true,
            ..Default::default()
        };

        let library = VulkanLibrary::new().expect("Vulkan library not found");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: Surface::required_extensions(&window.window_handle),
                ..Default::default()
            },
        )
        .unwrap();

        let surface = Surface::from_window(instance.clone(), window.window_handle.clone()).unwrap();
        let (physical_device, queue_family_index, transfer_family_index) =
            select_physical_device(instance.clone(), surface.clone(), &extensions, &features);

        debug!("Vulkan version: {}", instance.api_version());

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: {
                    if transfer_family_index.is_some() {
                        vec![
                            QueueCreateInfo {
                                queue_family_index: queue_family_index,
                                ..Default::default()
                            },
                            QueueCreateInfo {
                                queue_family_index: transfer_family_index.unwrap(),
                                ..Default::default()
                            },
                        ]
                    } else {
                        vec![QueueCreateInfo {
                            queue_family_index: queue_family_index,
                            ..Default::default()
                        }]
                    }
                },
                enabled_extensions: extensions,
                enabled_features: features,
                ..Default::default()
            },
        )
        .unwrap();
        let queue = queues.next().unwrap();
        let transfer_queue = queues.next();

        let memeory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 128,
                ..Default::default()
            },
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let (swapchain, images) = get_swapchain(
            window.window_handle.inner_size(),
            physical_device.clone(),
            device.clone(),
            surface.clone(),
        );
        let render_pass = get_render_pass(device.clone(), swapchain.clone());
        let framebuffers = get_framebuffers(device.clone(), &images, render_pass.clone());

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
                        memeory_allocator.clone(),
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
            library,
            instance,
            surface,
            physical_device,
            queue_family_index,
            transfer_family_index,
            device,
            queue,
            transfer_queue,
            memeory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
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
            dynamic_mesh_data: HashMap::new(),
        }
    }
}

pub struct RendererHandler {}

impl System for RendererHandler {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, world: &World, assets: &mut AssetLibrary, state: &mut State) {
        handle_possible_resize(world, assets, state);
        render(world, assets, state);
    }
}
