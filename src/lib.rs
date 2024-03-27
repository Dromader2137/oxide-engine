pub mod asset_library;
pub mod ecs;
pub mod input;
pub mod rendering;
pub mod state;
pub mod types;
pub mod utility;

use std::time::Instant;

use asset_library::AssetLibrary;
use ecs::World;
use input::InputManager;
use rendering::{EventLoop, Renderer, RendererHandler, Window};
use state::State;
use types::camera::CameraUpdater;
use types::mesh::{DynamicMeshLoader, MeshLoader};
use types::shader::ShaderLoader;
use types::texture::TextureLoader;
use types::transform::TransformUpdater;

use types::vectors::Vec2f;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;

pub fn run(mut world: World, mut assets: AssetLibrary) {
    let event_loop = EventLoop::new();
    let timer = Instant::now();
    let mut state = State {
        window: Window::new(&event_loop),
        input: InputManager::new(),
        renderer: Renderer::new(),
        time: 0.0,
        delta_time: 0.0
    };
    
    rendering::init(&mut state);
    
    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});
    world.add_system(MeshLoader {});
    world.add_system(DynamicMeshLoader {});
    world.add_system(ShaderLoader {});
    world.add_system(TextureLoader {});
    world.add_system(RendererHandler {});
    world.start(&mut assets, &mut state);

    event_loop.event_loop.set_control_flow(ControlFlow::Poll);
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
                state.input.mouse_pos += Vec2f::new([x as f32, y as f32]);
            }
            Event::AboutToWait => {
                let current_time = (timer.elapsed().as_millis() as f64) / 1000.0;
                state.delta_time = current_time - state.time;
                state.time = current_time;

                world.update(&mut assets, &mut state);

                state.input.clear_temp();
            }
            _ => (),
        })
        .unwrap();
}
