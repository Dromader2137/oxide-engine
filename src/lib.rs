pub mod asset_library;
pub mod asset_descriptions;
pub mod ecs;
pub mod input;
pub mod rendering;
pub mod state;
pub mod types;
pub mod loaders;
pub mod vulkan;
pub mod ui;
pub mod physics;
pub mod assets;

use std::fs;
use std::time::Instant;

use asset_descriptions::AssetDescriptions;
use ecs::World;
use input::{InputManager, InputManagerUpdater};
use log::trace;
use physics::collision_handler::CollisionHandler;
use physics::rigidbody::RigidbodyHandler;
use rendering::{EventLoop, Renderer, RendererHandler, Window};
use state::State;
use types::camera::CameraUpdater;
use types::material::MaterialLoader;
use types::mesh::{DynamicMeshMaterialLoader, MeshBufferLoader};
use types::model::ModelComponentUuidLoader;
use types::shader::ShaderLoader;
use types::texture::{DefaultTextureLoader, TextureLoader};
use types::transform::TransformUpdater;

use types::vectors::Vec2f;
use ui::ui_layout::{UiHandler, UiMeshBuilder};
use vulkan::context::VulkanContext;
use vulkan::memory::MemoryAllocators;
use winit::event::DeviceEvent::MouseMotion;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent::{self, MouseInput}};
use winit::event_loop::ControlFlow;

pub use winit;
pub use vulkano;
pub use hecs;
pub use log;
pub use uuid;
pub use image;

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
    let vulkan_context = VulkanContext::new(&window);
    let memory_allocators = MemoryAllocators::new(&vulkan_context);
    let renderer = Renderer::new(&vulkan_context, &memory_allocators, &window) ;
    let mut state = State {
        window,
        input: InputManager::new(),
        vulkan_context,
        memory_allocators,
        renderer,
        time: 0.0,
        delta_time: 0.0,
        physics_time_scale: 1.0
    };

    world.add_system(DynamicMeshMaterialLoader {});
    world.add_system(ModelComponentUuidLoader {});
    world.add_system(UiMeshBuilder {});

    world.add_system(TransformUpdater {});
    world.add_system(CameraUpdater {});

    world.add_system(MaterialLoader {});
    world.add_system(ShaderLoader {});
    world.add_system(TextureLoader {});
    world.add_system(MeshBufferLoader::new(&mut state));

    world.add_system(RendererHandler {});
    world.add_system(DefaultTextureLoader {});
    world.add_system(RigidbodyHandler {});
    world.add_system(CollisionHandler {});
    world.add_system(UiHandler {});
    world.add_system(InputManagerUpdater {});

    world.start(&mut assets, &mut state);

    event_loop.event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => {
                trace!("Close requested!");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_), ..
            } => {
                trace!("Resizing!");
                state.renderer.window_resized = true;
            }
            Event::WindowEvent {
                event: KeyboardInput {
                    event: KeyEvent {
                                logical_key: key_code,
                                state: ElementState::Pressed, ..
                    }, .. 
                }, ..
            } => {
                state.input.process_key_press(key_code);
            }
            Event::WindowEvent {
                event: KeyboardInput {
                        event: KeyEvent {
                                logical_key: key_code,
                                state: ElementState::Released, ..
                    }, ..
                }, ..
            } => {
                state.input.process_key_release(key_code);
            }
            Event::WindowEvent {
                event: MouseInput {
                    device_id: _,
                    state: ElementState::Pressed,
                    button
                }, ..
            } => {
                state.input.process_button_press(button);
            }
            Event::WindowEvent {
                event: MouseInput {
                    device_id: _,
                    state: ElementState::Released,
                    button
                }, ..
            } => {
                state.input.process_button_release(button);
            }
            Event::DeviceEvent {
                event: MouseMotion { delta: (x, y) }, ..
            } => {
                state.input.mouse_motion(Vec2f::new([x as f32, y as f32]));
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { 
                    delta, 
                    .. 
                }, 
                ..
            } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(val) => val.y as f32
                };
                state.input.scroll_delta = y;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    },
                ..
            } => {
                let logical_position = position.to_logical::<i32>(state.window.window_handle.scale_factor());
                let x = logical_position.x as f32;
                let y = logical_position.y as f32;
                state.input.cursor_position = Vec2f::new([x, y]);
            }
            Event::AboutToWait => {
                let current_time = (timer.elapsed().as_millis() as f64) / 1000.0;
                state.delta_time = current_time - state.time;
                state.time = current_time;

                world.update(&mut assets, &mut state);
            }
            _ => (),
        })
        .unwrap();
}
