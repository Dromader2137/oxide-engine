use std::collections::HashSet;

use winit::keyboard::Key;

#[derive(Clone, Debug)]
pub struct InputManager {
    pub pressed: HashSet<Key>,
    pub down: HashSet<Key>,
    pub released: HashSet<Key>,
}

impl InputManager {
    pub fn process_key_press(&mut self, key_code: Key) {
        let already_there = self.down.insert(key_code);
        if already_there {
            self.pressed.insert(key_code);
        }
    }

    pub fn process_key_release(&mut self, key_code: Key) {
        self.down.remove(&key_code);
        self.released.insert(key_code);
    }

    pub fn clear_temp(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn new() -> InputManager {
        InputManager { 
            pressed: HashSet::new(), 
            down: HashSet::new(), 
            released: HashSet::new() 
        }
    }
}
