pub struct VertexNode {
    id: u32
}
pub struct ColorNode {
    id: u32
}
pub struct OutputNode {
    id: u32
}

pub enum NodeOutput {
    Texture
}

pub enum NodeInput {
    Texture
}

pub trait RendererNode {
    fn set_id(&mut self, id: u32);
    fn get_id(&self) -> u32;
    fn get_outputs(&self) -> Vec<NodeOutput> { vec![] }
    fn get_inputs(&self) -> Vec<NodeInput> { vec![] }
}

impl RendererNode for VertexNode {
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_outputs(&self) -> Vec<NodeOutput> {
        
    }
}
