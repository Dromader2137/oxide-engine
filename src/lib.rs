pub mod asset_library;
pub mod ecs;
pub mod input;
pub mod rendering;
pub mod state;
pub mod types;
pub mod utility;
pub mod asset_descriptions;
pub mod loaders;

use std::fs;
use std::time::Instant;

use asset_descriptions::AssetDescriptions;
use ecs::World;
use input::InputManager;
use log::trace;
use rendering::{EventLoop, Renderer, RendererHandler, Window};
use state::State;
use types::camera::CameraUpdater;
use types::material::MaterialLoader;
use types::mesh::MeshBufferLoader;
use types::shader::ShaderLoader;
use types::texture::{DefaultTextureLoader, TextureLoader};
use types::transform::TransformUpdater;

use types::vectors::Vec2f;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;

pub fn run(mut world: World, asset_descriptions: AssetDescriptions) {
    env_logger::init();
    let timer = Instant::now();

    let mut assets = if cfg!(feature = "dev_tools") {
        log::debug!("Recreating asset pack...");
        let mut assets = asset_descriptions.generate_library();
        types::mesh::load_model_meshes(&mut assets);
        let _ = std::fs::write("assets.data", rmp_serde::to_vec(&assets).unwrap());
        assets
    } else {
        rmp_serde::from_slice(fs::read("assets.data").unwrap().as_slice()).unwrap()
    };
        
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop);
    let mut state = State {
        window: window.clone(),
        input: InputManager::new(),
        renderer: Renderer::new(&window),
        time: 0.0,
        delta_time: 0.0
    };

    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});
    world.add_system(MaterialLoader {});
    world.add_system(ShaderLoader {});
    world.add_system(TextureLoader {});
    world.add_system(DefaultTextureLoader {});
    world.add_system(MeshBufferLoader {});
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
                trace!("Close requested!");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                trace!("Resizing!");
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
