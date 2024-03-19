pub mod asset_library;
pub mod ecs;
pub mod input;
pub mod rendering;
pub mod state;
pub mod types;
pub mod utility;

use asset_library::AssetLibrary;
use ecs::World;
use input::InputManager;
use rendering::{
    handle_possible_resize, render, update_command_buffers, wait_for_idle, CameraUpdater, EventLoop, MeshLoader, Renderer, ShaderLoader, Window
};
use state::State;
use types::transform::TransformUpdater;

use types::vectors::Vec2f;
use winit::dpi::LogicalPosition;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::CursorGrabMode;

pub fn run(mut world: World, mut assets: AssetLibrary) {
    let event_loop = EventLoop::new();
    let mut state = State {
        window: Window::new(&event_loop),
        input: InputManager::new(),
        renderer: Renderer::new(),
    };
    
    rendering::init(&mut state);

    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});
    world.add_system(MeshLoader {});
    world.add_system(ShaderLoader {});
    world.start(&mut assets, &mut state);

    update_command_buffers(&mut world, &assets, &mut state);

    event_loop.event_loop.set_control_flow(ControlFlow::Poll);
    state
        .window
        .window_handle
        .set_cursor_grab(CursorGrabMode::Locked)
        .unwrap();
    state.window.window_handle.set_cursor_visible(false);
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
                state.renderer.window_resized = true;
            }
            Event::WindowEvent {
                event:
                    KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: key_code,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                state.input.process_key_press(key_code);
            }
            Event::WindowEvent {
                event:
                    KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: key_code,
                                state: ElementState::Released,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                state.input.process_key_release(key_code);
            }
            Event::DeviceEvent {
                event: MouseMotion { delta: (x, y) },
                ..
            } => {
                state.input.mouse_pos = state.input.mouse_pos + Vec2f::new([x as f32, y as f32]);
            }
            Event::AboutToWait => {
                handle_possible_resize(&mut world, &assets, &mut state);
                render(&mut state);
                wait_for_idle(&mut state);

                world.update(&mut assets, &mut state);

                state.input.clear_temp();
                let size = state.window.window_handle.inner_size();
                state
                    .window
                    .window_handle
                    .set_cursor_position(LogicalPosition::new(size.width / 2, size.height / 2))
                    .unwrap();
            }
            _ => (),
        })
        .unwrap();
}
