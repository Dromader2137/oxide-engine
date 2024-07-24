use std::{collections::HashMap, time::Instant};

use log::debug;
use uuid::Uuid;

use crate::{asset_library::AssetLibrary, state::State};

pub trait System {
    fn on_start(&self, world: &World, assets: &mut AssetLibrary, state: &mut State);
    fn on_update(&self, world: &World, assets: &mut AssetLibrary, state: &mut State);
}

pub trait Callback {
    fn action(&self, world: &World, assets: &mut AssetLibrary, state: &mut State);
}

pub struct World {
    pub entities: hecs::World,
    pub systems: Vec<Box<dyn System>>,
    pub callbacks: HashMap<Uuid, Box<dyn Callback>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: hecs::World::new(),
            systems: Vec::new(),
            callbacks: HashMap::new()
        }
    }

    pub fn add_system<SystemType: 'static + System>(&mut self, system: SystemType) {
        self.systems.push(Box::new(system));
    }

    pub fn add_callback<T: 'static + Callback>(&mut self, callback: T) -> Uuid {
        let uuid = Uuid::new_v4();
        self.callbacks.insert(uuid, Box::new(callback));
        uuid
    }

    pub fn start(&mut self, assets: &mut AssetLibrary, state: &mut State) {
        for system in self.systems.iter() {
            system.on_start(self, assets, state);
        }
    }

    pub fn update(&mut self, assets: &mut AssetLibrary, state: &mut State) {
        let mut timer = Instant::now();
        let mut i = 0;
        for system in self.systems.iter() {
            system.on_update(self, assets, state);
            debug!("{}: {}", i, timer.elapsed().as_millis());
            i += 1;
            timer = Instant::now();
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
