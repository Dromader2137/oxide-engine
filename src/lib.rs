pub mod ecs;
pub mod rendering;
pub mod types;
pub mod utility;

use ecs::World;
use rendering::{EventLoop, Renderer, ShaderManager, Window, CameraUpdater, MeshUpdater};
use types::transform::TransformUpdater;

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

pub fn run(mut world: World) {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop);
    let mut renderer: Renderer = Renderer::new();
    renderer.init(&event_loop, &window);
    
    let mut shader_manager = ShaderManager::new();
    shader_manager.load(&mut renderer);

    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});
    world.add_system(MeshUpdater {});
    world.start(&mut renderer);

    renderer.update_command_buffers(&mut world, &shader_manager);
    event_loop
        .event_loop
        .run(move |event, _, control_flow| match event {
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
                renderer.handle_possible_resize(&window,  &mut world, &mut shader_manager);
                renderer.render();
                renderer.wait_for_idle();
                world.update(&mut renderer);
            }
            _ => (),
        });
}
