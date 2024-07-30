use std::collections::HashSet;

use winit::{event::MouseButton, keyboard::Key};

use crate::{
    asset_library::AssetLibrary,
    ecs::{System, World},
    state::State,
    types::vectors::Vec2f,
};

#[derive(Clone, Debug)]
pub struct InputManager {
    pub key_pressed: HashSet<Key>,
    pub key_down: HashSet<Key>,
    pub key_released: HashSet<Key>,
    
    pub button_pressed: HashSet<MouseButton>,
    pub button_down: HashSet<MouseButton>,
    pub button_released: HashSet<MouseButton>,

    pub cursor_position: Vec2f,
    pub scroll_delta: f32,

    mouse_pos: Vec2f,
    prev_mouse_pos: Option<Vec2f>,
}

impl InputManager {
    pub fn process_key_press(&mut self, mut key_code: Key) {
        if let Key::Character(key) = key_code {
            key_code = Key::Character(key.to_lowercase().into());
        }
        let already_there = self.key_down.insert(key_code.clone());
        if already_there {
            self.key_pressed.insert(key_code);
        }
    }

    pub fn process_key_release(&mut self, mut key_code: Key) {
        if let Key::Character(key) = key_code {
            key_code = Key::Character(key.to_lowercase().into());
        }
        self.key_down.remove(&key_code);
        self.key_released.insert(key_code);
    }
    
    pub fn process_button_press(&mut self, button: MouseButton) {
        let already_there = self.button_down.insert(button);
        if already_there {
            self.button_pressed.insert(button);
        }
    }

    pub fn process_button_release(&mut self, button: MouseButton) {
        self.button_down.remove(&button);
        self.button_released.insert(button);
    }

    pub fn get_mouse_delta(&self) -> Vec2f {
        if self.prev_mouse_pos.is_none() {
            Vec2f::new([0.0, 0.0])
        } else {
            Vec2f::new([
                self.mouse_pos.x - self.prev_mouse_pos.unwrap().x,
                self.mouse_pos.y - self.prev_mouse_pos.unwrap().y,
            ])
        }
    }

    pub fn clear_temp(&mut self) {
        self.key_pressed.clear();
        self.key_released.clear();
        self.button_pressed.clear();
        self.button_released.clear();
        self.prev_mouse_pos = Some(self.mouse_pos);
        self.scroll_delta = 0.0;
    }

    pub fn new() -> InputManager {
        InputManager {
            key_pressed: HashSet::new(),
            key_down: HashSet::new(),
            key_released: HashSet::new(),
            button_down: HashSet::new(),
            button_pressed: HashSet::new(),
            button_released: HashSet::new(), 
            cursor_position: Vec2f::new([0.0, 0.0]),
            scroll_delta: 0.0,
            mouse_pos: Vec2f::new([0.0, 0.0]),
            prev_mouse_pos: None,
        }
    }

    pub fn mouse_motion(&mut self, delta: Vec2f) {
        self.mouse_pos += delta;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputManagerUpdater {}

impl System for InputManagerUpdater {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, state: &mut State) {
        state.input.clear_temp();
    }
}
