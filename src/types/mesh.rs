use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::VertexData, state::State};

pub struct DynamicMesh {
    pub vertices: Vec<VertexData>,
    pub material: String,
    pub buffer_id: Option<u32>,
    pub changed: bool
}

impl DynamicMesh {
    pub fn new(vertices: Vec<VertexData>, material: String) -> Self {
        Self { vertices, material, buffer_id: None, changed: false }
    }
}

pub struct DynamicMeshLoader {}

impl System for DynamicMeshLoader {
    fn on_start(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
