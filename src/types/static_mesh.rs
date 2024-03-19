use crate::state::State;

#[derive(Clone, Debug)]
pub struct StaticMesh {
    pub mesh_name: String
}

impl StaticMesh {
    pub fn set_mesh(&mut self, state: &mut State, name: String) {
        self.mesh_name = name;
        state.renderer.command_buffer_outdated = true;
    }
}
