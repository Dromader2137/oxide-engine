use crate::{
    input::InputManager,
    rendering::{Renderer, Window}, ui::uimanager::UiStorage,
};

pub struct State {
    pub window: Window,
    pub input: InputManager,
    pub renderer: Renderer,
    pub ui: UiStorage,
    pub time: f64,
    pub delta_time: f64
}
