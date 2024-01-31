pub mod types;
pub mod rendering;
pub mod ecs;
pub mod utility;

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use ecs::World;
use rendering::{Renderer, Window, EventLoop, ShaderManager, Mesh, ShaderData, ModelData, VPData, ShaderType};
use types::buffers::UpdatableBuffer;
use types::transform::Transform;
use utility::read_file_to_words;
use vulkano::buffer::BufferUsage;

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use types::vectors::*;
use types::matrices::*;

pub fn run(mut world: World) {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop); 
    let mut renderer: Renderer = Renderer::new();
    let mut shader_manager = ShaderManager::new();

    renderer.init(&event_loop, &window);

    let mut vp_buffer: UpdatableBuffer<VPData> = UpdatableBuffer::new(&renderer.device.clone().unwrap().clone(), BufferUsage::UNIFORM_BUFFER);
    let mut vp_data = VPData {
        view: Matrix4f::look_at(Vec3f::new([0.0, 0.0, 0.0]), Vec3f::new([0.0, 0.0, 1.0]), Vec3f::new([0.0, 1.0, 0.0])),
        projection: Matrix4f::perspective((75.0_f32).to_radians(), 1.0, 0.1, 10.0)
    };
    vp_buffer.write(vp_data);
    
    let mut shaders: HashMap<String, ShaderData> = HashMap::new();
    for file in fs::read_dir("./shaders/bin/").unwrap() {
        let path = file.unwrap().path().to_str().unwrap().to_string();
        let processed_path = path.split_once("/").unwrap().1.split_once("/").unwrap().1.split_once("/").unwrap().1.split_once(".").unwrap().0.trim();
        if processed_path == "" { continue; }
        let shader_type = if processed_path != "simple" { ShaderType::Fragment } else { ShaderType::Vertex };
        shaders.insert(
            processed_path.to_string(),
            ShaderData {
                shader_code: read_file_to_words(path.split_once("/").unwrap().1), 
                shader_type
        });
    }
    shader_manager.load(&mut renderer, shaders);

    let mut now = Instant::now();
    let mut dbg = 0.0;

    for mesh in world.borrow_component_vec_mut::<Mesh>().unwrap().iter_mut() {
        mesh.as_mut().unwrap().load(&mut renderer);
    }
    for transform in world.borrow_component_vec_mut::<Transform>().unwrap().iter_mut() {
        transform.as_mut().unwrap().load(&renderer);
        let position = transform.as_ref().unwrap().position;
        transform.as_mut().unwrap().buffer.as_mut().unwrap().write(Matrix4f::translation(position));
    }
    renderer.update_command_buffers(&mut world, &shader_manager, &vp_buffer);

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
            renderer.handle_possible_resize(&window, &mut vp_data, &vp_buffer, &mut world, &mut shader_manager);
            renderer.render();
            renderer.wait_for_idle();

            dbg += now.elapsed().as_secs_f32();
            now = Instant::now();
            vp_data.view = Matrix4f::look_at(Vec3f::new([0.0, 0.0, 0.0]), Vec3f::new([dbg.sin(), 0.0, dbg.cos()]), Vec3f::new([0.0, 1.0, 0.0]));
            vp_buffer.write(vp_data);
        }
        _ => (),
    });
}
