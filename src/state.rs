use crate::{
    input::InputManager,
    rendering::{Renderer, Window},
};

pub struct State {
    pub window: Window,
    pub input: InputManager,
    pub renderer: Renderer,
    pub time: f64,
    pub delta_time: f64
}
