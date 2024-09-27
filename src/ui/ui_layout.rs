use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use winit::event::MouseButton;

use crate::{ecs::System, state::State, types::vectors::Vec2f};

use super::ui_mesh::UiMesh;

#[derive(Pod, Zeroable, Clone, Copy, Debug, Serialize, Deserialize, Vertex)]
#[repr(C)]
pub struct UiVertexData {
    #[format(R32G32B32A32_SFLOAT)]
    pub position: Vec2f,
    #[format(R32G32B32A32_SFLOAT)]
    pub uv: Vec2f,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Anchor {
    Center,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight
}

fn anchor_to_offset(anchor: Anchor) -> Vec2f {
    match anchor {
        Anchor::Center => Vec2f::new([0.0, 0.0]),
        Anchor::Up => Vec2f::new([0.0, -1.0]),
        Anchor::Down => Vec2f::new([0.0, 1.0]),
        Anchor::Left => Vec2f::new([-1.0, 0.0]),
        Anchor::Right => Vec2f::new([1.0, 0.0]),
        Anchor::UpLeft => Vec2f::new([-1.0, -1.0]),
        Anchor::UpRight => Vec2f::new([1.0, -1.0]),
        Anchor::DownLeft => Vec2f::new([-1.0, 1.0]),
        Anchor::DownRight => Vec2f::new([1.0, 1.0]),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum UiElementType {
    None,
    Button(Uuid)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiElement {
    pub element_type: UiElementType,
    pub name: String,
    pub material: Uuid,
    position: Vec2f,
    screen_anchor: Anchor,
    width: f32,
    height: f32,
    pub mesh: Option<UiMesh>,
}

impl UiElement {
    pub fn new(name: &str, element_type: UiElementType, material: Uuid, screen_anchor: Anchor, position: Vec2f, width: f32, height: f32) -> UiElement {
        UiElement { name: name.to_string(), element_type, material, screen_anchor, position, width, height, mesh: None }
    }

    pub fn generate_mesh(&self, state: &State) -> UiMesh {
        let offset = anchor_to_offset(self.screen_anchor);
        let x_offset = self.width / 2.0;
        let y_offset = self.height / 2.0;
        let window_size = state.window.window_handle.inner_size();
        let ratio = window_size.width as f32 / window_size.height as f32;

        let v1 = UiVertexData {position: Vec2f::new([self.position.x + offset.x - x_offset, -self.position.y * ratio + offset.y - y_offset * ratio]), uv: Vec2f::new([0.0, 0.0])};
        let v2 = UiVertexData {position: Vec2f::new([self.position.x + offset.x - x_offset, -self.position.y * ratio + offset.y + y_offset * ratio]), uv: Vec2f::new([0.0, 1.0])};
        let v3 = UiVertexData {position: Vec2f::new([self.position.x + offset.x + x_offset, -self.position.y * ratio + offset.y + y_offset * ratio]), uv: Vec2f::new([1.0, 1.0])};
        let v4 = UiVertexData {position: Vec2f::new([self.position.x + offset.x + x_offset, -self.position.y * ratio + offset.y - y_offset * ratio]), uv: Vec2f::new([1.0, 0.0])};
        let vertices = vec![v1, v2, v3, v4];
        let indices = vec![0, 1, 2, 0, 2, 3];
        UiMesh::new(vertices, indices)
    }
}

impl Default for UiElement {
    fn default() -> Self {
        UiElement::new("UiLayer", UiElementType::None, Uuid::nil(), Anchor::Center, Vec2f::new([0.0, 0.0]), 0.2, 0.2)
    }
}

pub struct UiMeshBuilder {}

impl System for UiMeshBuilder {
    fn on_start(&self, _world: &crate::ecs::World, assets: &mut crate::assets::asset_library::AssetLibrary, state: &mut crate::state::State) {
        for (_, element) in assets.ui.iter_mut() {
            let mut mesh = element.generate_mesh(state);
            mesh.load(state);
            element.mesh = Some(mesh);
        }
    }

    fn on_update(&self, _world: &crate::ecs::World, assets: &mut crate::assets::asset_library::AssetLibrary, state: &mut crate::state::State) {
        if !state.renderer.window_resized { return; }
        for (_, element) in assets.ui.iter_mut() {
            let mut mesh = element.generate_mesh(state);
            mesh.load(state);
            element.mesh = Some(mesh);
        }
    }
}

pub struct UiHandler {}

impl System for UiHandler {
    fn on_start(&self, _world: &crate::ecs::World, _assets: &mut crate::assets::asset_library::AssetLibrary, _state: &mut State) {}

    fn on_update(&self, world: &crate::ecs::World, assets: &mut crate::assets::asset_library::AssetLibrary, state: &mut State) {
        let window_size = Vec2f::new([state.window.window_handle.inner_size().width as f32, state.window.window_handle.inner_size().height as f32]);
        let normalized_position = (state.input.cursor_position / window_size - Vec2f::new([0.5, 0.5])) * 2.0;

        for i in 0..assets.ui.len() {
            let ui_element = assets.ui.values().nth(i).unwrap();
            match ui_element.element_type {
                UiElementType::Button(uuid) => {
                    let offset = anchor_to_offset(ui_element.screen_anchor);
                    let x_offset = ui_element.width / 2.0;
                    let y_offset = ui_element.height / 2.0;
                    let window_size = state.window.window_handle.inner_size();
                    let ratio = window_size.width as f32 / window_size.height as f32;

                    let left =  ui_element.position.x         + offset.x - x_offset; 
                    let right = ui_element.position.x         + offset.x + x_offset; 
                    let up =   -ui_element.position.y * ratio + offset.y - y_offset * ratio;
                    let down = -ui_element.position.y * ratio + offset.y + y_offset * ratio;

                    let hit = normalized_position.x >= left && normalized_position.x <= right && normalized_position.y >= up && normalized_position.y <= down;
                    if hit && state.input.button_pressed.contains(&MouseButton::Left) {
                        world.callbacks.get(&uuid).expect("Callback not found").action(world, assets, state);
                    }
                },
                UiElementType::None => {}
            }
        }
    }
}
