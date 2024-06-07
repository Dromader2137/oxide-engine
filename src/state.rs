use std::collections::HashMap;

use crate::{
    input::InputManager, rendering::{Renderer, Window}, types::mesh::Mesh
};

pub struct State {
    pub window: Window,
    pub input: InputManager,
    pub renderer: Renderer,
    pub meshes: HashMap<String, Mesh>,
    pub time: f64,
    pub delta_time: f64
}
