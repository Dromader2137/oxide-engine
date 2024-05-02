use crate::{asset_library::AssetLibrary, state::State};

pub trait System {
    fn on_start(&self, world: &World, assets: &mut AssetLibrary, state: &mut State);
    fn on_update(&self, world: &World, assets: &mut AssetLibrary, state: &mut State);
}

pub struct World {
    pub entities: hecs::World,
    pub systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: hecs::World::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_system<SystemType: 'static + System>(&mut self, system: SystemType) {
        self.systems.push(Box::new(system));
    }

    pub fn start(&mut self, assets: &mut AssetLibrary, state: &mut State) {
        for system in self.systems.iter() {
            system.on_start(self, assets, state);
        }
    }

    pub fn update(&mut self, assets: &mut AssetLibrary, state: &mut State) {
        for system in self.systems.iter() {
            system.on_update(self, assets, state);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
