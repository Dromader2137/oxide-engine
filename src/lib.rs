pub mod ecs;
pub mod rendering;
pub mod types;
pub mod utility;
pub mod input;

use ecs::World;
use input::InputManager;
use rendering::{EventLoop, Renderer, ShaderManager, Window, CameraUpdater, MeshUpdater};
use types::transform::TransformUpdater;

use winit::event::{Event, WindowEvent};

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
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Exit requested!");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                println!("Resizing!");
                renderer.window_resized = true;
            }
            // Event::WindowEvent { 
            //     event: WindowEvent::KeyboardInput { 
            //         input: Keybo { 
            //             virtual_keycode: Some(key),
            //             state: ElementState::Pressed,
            //             ..
            //         },
            //         ..
            //     },
            //     ..
            // } => {
            //     let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
            //     let mut input_manager = input_manager_list.iter_mut();
            //     input_manager.next().unwrap().as_mut().unwrap().process_key_press(key);
            // }
            // Event::WindowEvent { 
            //     event: WindowEvent::KeyboardInput { 
            //         input: KeyboardInput { 
            //             virtual_keycode: Some(key),
            //             state: ElementState::Released,
            //             ..
            //         },
            //         ..
            //     },
            //     ..
            // } => {
            //     let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
            //     let mut input_manager = input_manager_list.iter_mut();
            //     input_manager.next().unwrap().as_mut().unwrap().process_key_release(key);
            // }
            Event::AboutToWait => {
                renderer.handle_possible_resize(&window,  &mut world, &mut shader_manager);
                renderer.render();
                renderer.wait_for_idle();
                world.update(&mut renderer);
                let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
                let input_manager = input_manager_list.iter_mut().next().unwrap().as_mut().unwrap();
                input_manager.clear_temp();
            }
            _ => (),
        }
    );
}
