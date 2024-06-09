

#[derive(Debug)]
pub enum Attachment {
    Texture(String)
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub attachments: Vec<Attachment>
}
