use std::collections::HashSet;

use winit::keyboard::Key;

use crate::{asset_library::AssetLibrary, ecs::{System, World}, state::State, types::vectors::Vec2f};

#[derive(Clone, Debug)]
pub struct InputManager {
    pub pressed: HashSet<Key>,
    pub down: HashSet<Key>,
    pub released: HashSet<Key>,

    pub mouse_pos: Vec2f,
    prev_mouse_pos: Option<Vec2f>,
}

impl InputManager {
    pub fn process_key_press(&mut self, key_code: Key) {
        let already_there = self.down.insert(key_code.clone());
        if already_there {
            self.pressed.insert(key_code);
        }
    }

    pub fn process_key_release(&mut self, key_code: Key) {
        self.down.remove(&key_code);
        self.released.insert(key_code);
    }

    pub fn get_mouse_delta(&self) -> Vec2f {
        if self.prev_mouse_pos.is_none() { 
            Vec2f::new([0.0, 0.0])
        } else { 
            Vec2f::new([self.mouse_pos.x - self.prev_mouse_pos.unwrap().x, self.mouse_pos.y - self.prev_mouse_pos.unwrap().y])
        }
    }

    pub fn clear_temp(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.prev_mouse_pos = Some(self.mouse_pos.clone());
    }

    pub fn new() -> InputManager {
        InputManager { 
            pressed: HashSet::new(), 
            down: HashSet::new(), 
            released: HashSet::new(),
            mouse_pos: Vec2f::new([0.0, 0.0]),
            prev_mouse_pos: None
        }
    }
}

struct InputManagerUpdater {}

impl System for InputManagerUpdater {
    fn on_start(&self, _world: &World, _assets: &AssetLibrary, _state: &State) {}
    fn on_update(&self, _world: &World, _assets: &AssetLibrary, state: &State) {
        state.input.clear_temp();
    }
}
