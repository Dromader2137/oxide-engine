use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::time::Instant;

use rendering::{Renderer, Window, EventLoop, ShaderManager, MeshManager, Mesh, ShaderData, ModelData, VPData};
use types::buffers::UpdatableBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::shader::spirv::bytes_to_words;
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


pub fn run(meshes: Vec<Mesh>, shaders: HashMap<String, ShaderData>) {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop); 
    let mut renderer: Renderer = Renderer::new();
    let mut shader_manager = ShaderManager::new();
    let mut mesh_manager = MeshManager::new();
    
    renderer.init(&event_loop, &window);

    let mut model_buffers: Vec<UpdatableBuffer<ModelData>> = Vec::new();
    for _ in (0..2).step_by(1) {
        model_buffers.push(UpdatableBuffer::new(&renderer.device.clone().unwrap().clone(), BufferUsage::UNIFORM_BUFFER));
    }
    let mut vp_buffer: UpdatableBuffer<VPData> = UpdatableBuffer::new(&renderer.device.clone().unwrap().clone(), BufferUsage::UNIFORM_BUFFER);
    
    let mut model_data = vec![ModelData { translation: Matrix4f::translation(Vec3f::new([0.0, 0.0, -1.0]))}; 2];
    let mut vp_data = VPData {
        view: Matrix4f::look_at(Vec3f::new([0.0, 0.0, 0.0]), Vec3f::new([0.0, 0.0, 1.0]), Vec3f::new([0.0, 1.0, 0.0])),
        projection: Matrix4f::perspective((75.0_f32).to_radians(), 1.0, 0.1, 10.0)
    };
    
    for i in (0..2).step_by(1) {
        model_buffers.last_mut().unwrap().write(model_data[i as usize]);
    }
    vp_buffer.write(vp_data);

    shader_manager.load(&mut renderer, shaders);
    mesh_manager.load(&mut renderer, meshes);
    renderer.update_command_buffers(&mesh_manager, &shader_manager, &model_buffers, &vp_buffer);

    let mut now = Instant::now();
    let mut dbg = 0.0;

    event_loop.event_loop.run(move |event, _, control_flow| match event {
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
            renderer.handle_possible_resize(&window, &mut vp_data, &vp_buffer, &model_buffers, &mesh_manager, &mut shader_manager);

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

            dbg += now.elapsed().as_secs_f32();
            now = Instant::now();

            vp_data.view = Matrix4f::look_at(Vec3f::new([0.0, 0.0, 0.0]), Vec3f::new([dbg.sin(), 0.0, dbg.cos()]), Vec3f::new([0.0, 1.0, 0.0]));
            model_data[0].translation = Matrix4f::translation(Vec3f::new([0.0, 0.0, -2.0]));
            model_data[1].translation = Matrix4f::translation(Vec3f::new([0.0, 0.0, 2.0]));
            vp_buffer.write(vp_data);
            for (i, model_buffer) in model_buffers.iter_mut().enumerate() {
                model_buffer.write(model_data[i]);
            }

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
