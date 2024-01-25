use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use rendering::{SimpleRenderer, Window, EventLoop, ShaderManager, MeshManager, Mesh, ShaderData, ModelData, VPData, ShaderType, Shader};
use types::buffers::UpdatableBuffer;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::shader::spirv::bytes_to_words;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain::{self, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

pub mod types;
use types::vectors::*;
use types::matrices::*;
pub mod rendering;

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

pub fn run(mut meshes: Vec<Mesh>, shaders: HashMap<String, ShaderData>) {
    let event_loop_ = EventLoop::new();
    let window_ = Window::new(&event_loop_); 
    let mut renderer: SimpleRenderer = SimpleRenderer::new();
    renderer.init(&event_loop_, &window_);
    let mut shader_manager = ShaderManager::new();
    let mut mesh_manager = MeshManager::new();

    for mesh in meshes.iter_mut() {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(renderer.device.clone().unwrap().clone()));
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
    mesh_manager.list = meshes;

    let mut model_buffers: Vec<UpdatableBuffer<ModelData>> = Vec::new();
    for _ in (0..2).step_by(1) {
        model_buffers.push(
            UpdatableBuffer::new(&renderer.device.clone().unwrap().clone(), BufferUsage::UNIFORM_BUFFER)
        );
        model_buffers.last_mut().unwrap().write(ModelData { translation: Matrix4f::translation(Vec3f::new([0.0, 0.0, -1.0]))});
    }

    let mut vp_data = VPData {
        view: Matrix4f::look_at(
                  Vec3f::new([0.0, 0.0, 0.0]), 
                  Vec3f::new([0.0, 0.0, 1.0]), 
                  Vec3f::new([0.0, 1.0, 0.0])),
        projection: Matrix4f::perspective((60.0_f32).to_radians(), 1.0, 0.1, 10.0)
    };
    let mut vp_buffer: UpdatableBuffer<VPData> = 
        UpdatableBuffer::new(&renderer.device.clone().unwrap().clone(), BufferUsage::UNIFORM_BUFFER);
    vp_buffer.write(vp_data);

    let vertex_shaders: Vec<(&String, &ShaderData)> = shaders
        .iter()
        .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Vertex))
        .collect();

    let fragment_shaders: Vec<(&String, &ShaderData)> = shaders
        .iter()
        .filter(|shader_data| matches!(shader_data.1.shader_type, ShaderType::Fragment))
        .collect();

    for (shader, _) in shaders.iter() {
        shader_manager.library.insert(
            shader.to_string(),
            Shader { shader: load_shader_module(&shaders, &renderer.device.clone().unwrap(), shader) },
        );
    }

    for (name_vert, _) in vertex_shaders.iter() {
        for (name_frag, _) in fragment_shaders.iter() {
            shader_manager.pipelines.insert(
                (name_vert.to_string(), name_frag.to_string()),
                renderer.get_pipeline(
                    shader_manager.library.get(*name_vert).unwrap(),
                    shader_manager.library.get(*name_frag).unwrap(),
                ),
            );
        }
    }

    renderer.update_command_buffers(&mesh_manager, &shader_manager, &model_buffers, &vp_buffer);

    let mut dbg: f32 = 0.0;

    event_loop_.event_loop.run(move |event, _, control_flow| match event {
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
            renderer.window_resized = true;
        }
        Event::MainEventsCleared => {
            renderer.handle_possible_resize(&window_, &mut vp_data, &vp_buffer, &model_buffers, &mesh_manager, &mut shader_manager);

            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(renderer.swapchain.as_ref().unwrap().clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        renderer.recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

            if suboptimal {
                renderer.recreate_swapchain = true;
            }

            if let Some(image_fence) = &renderer.fences.as_ref().unwrap()[image_i as usize] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match renderer.fences.as_ref().unwrap()[renderer.previous_fence].clone() {
                None => {
                    let mut now = sync::now(renderer.device.as_ref().unwrap().clone());
                    now.cleanup_finished();
                    now.boxed()
                }
                Some(fence) => fence.boxed(),
            };
            
            // Waiting for all fences to be able to write to buffers
            for fence in renderer.fences.as_mut().unwrap().iter_mut() {
                let _ = match fence.as_mut() {
                    Some(val) => val.wait(None).unwrap(),
                    _ => (), 
                };
            }

            vp_data.view = Matrix4f::look_at(Vec3f::new([0.0, 0.0, 0.0]), Vec3f::new([(dbg/5.0).sin(), 0.0, (dbg/5.0).cos()]), Vec3f::new([0.0, 1.0, 0.0]));
            model_buffers.get_mut(0).unwrap().write(
                ModelData { translation: Matrix4f::translation(Vec3f::new([0.0, 0.0, -5.0])) }
                );
            model_buffers.get_mut(1).unwrap().write(
                ModelData { translation: Matrix4f::translation(Vec3f::new([0.0, 0.0, 5.0])) }
                );
            vp_buffer.write(vp_data);
            dbg += 0.01;

            let future = previous_future
                .join(acquire_future)
                .then_execute(renderer.queue.as_ref().unwrap().clone(), renderer.command_buffers.as_ref().unwrap()[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    renderer.queue.as_ref().unwrap().clone(),
                    SwapchainPresentInfo::swapchain_image_index(renderer.swapchain.as_ref().unwrap().clone(), image_i),
                )
                .then_signal_fence_and_flush();

            renderer.fences.as_mut().unwrap()[image_i as usize] = match future.map_err(Validated::unwrap) {
                Ok(value) => Some(Arc::new(value)),
                Err(VulkanError::OutOfDate) => {
                    renderer.recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("failed to flush future: {e}");
                    None
                }
            };
            renderer.previous_fence = image_i as usize;
        }
        _ => (),
    });
}
