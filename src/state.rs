use crate::{
    input::InputManager,
    rendering::{Renderer, Window},
};

pub struct State {
    pub window: Window,
    pub input: InputManager,
    pub renderer: Renderer,
}
