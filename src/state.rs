use crate::{input::InputManager, rendering::{EventLoop, Window}};

pub struct State {
    pub window: Window,
    pub event_loop: EventLoop,
    pub input: InputManager
}
