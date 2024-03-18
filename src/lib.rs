pub mod ecs;
pub mod rendering;
pub mod types;
pub mod utility;
pub mod input;
pub mod asset_library;
pub mod state;

use asset_library::AssetLibrary;
use ecs::World;
use input::InputManager;
use rendering::{EventLoop, Renderer, Window, CameraUpdater, MeshUpdater};
use state::State;
use types::transform::TransformUpdater;

use winit::dpi::LogicalPosition;
use winit::event::{Event, WindowEvent, KeyEvent, ElementState};
use winit::event_loop::ControlFlow;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::DeviceEvent::MouseMotion;
use winit::window::CursorGrabMode;

pub fn run(mut world: World, mut assets: AssetLibrary) {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop);
    let input_manager = InputManager::new();
    let mut state = State {window, event_loop, input: input_manager};
    assets.renderer.init(&state);

    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});
    world.add_system(MeshUpdater {});
    world.start(&assets, &state);

    assets.renderer.update_command_buffers(&mut world, &assets);

    event_loop.event_loop.set_control_flow(ControlFlow::Poll);
    window.window_handle.set_cursor_grab(CursorGrabMode::Locked).unwrap();
    window.window_handle.set_cursor_visible(false);
    event_loop
        .event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Close requested!");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                println!("Resizing!");
                assets.renderer.window_resized = true;
            }
            Event::WindowEvent { 
                event: KeyboardInput {
                    event: KeyEvent { 
                        logical_key: key_code,
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                },
                ..
            } => {
                let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
                let mut input_manager = input_manager_list.iter_mut();
                input_manager.next().unwrap().as_mut().unwrap().process_key_press(key_code);
            }
            Event::WindowEvent { 
                event: KeyboardInput {
                    event: KeyEvent { 
                        logical_key: key_code,
                        state: ElementState::Released,
                        ..
                    },
                    ..
                },
                ..
            } => {
                let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
                let mut input_manager = input_manager_list.iter_mut();
                input_manager.next().unwrap().as_mut().unwrap().process_key_release(key_code);
            }
            Event::DeviceEvent { 
                event: MouseMotion {
                    delta: (x, y)
                },
                ..
            } => {
                let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
                let input_manager = input_manager_list.iter_mut().next().unwrap().as_mut().unwrap();
                input_manager.mouse_pos.x += x as f32;
                input_manager.mouse_pos.y += y as f32;
            }
            Event::AboutToWait => {
                assets.renderer.handle_possible_resize(&mut world, &assets, &state);
                assets.renderer.render();
                assets.renderer.wait_for_idle();
                
                world.update(&assets, &state);
                
                let mut input_manager_list = world.borrow_component_vec_mut::<InputManager>().unwrap();
                let input_manager = input_manager_list.iter_mut().next().unwrap().as_mut().unwrap();
                input_manager.clear_temp();
                let size = window.window_handle.inner_size();
                window.window_handle.set_cursor_position(LogicalPosition::new(size.width / 2, size.height / 2)).unwrap();
            }
            _ => (),
        }
    ).unwrap();
}
