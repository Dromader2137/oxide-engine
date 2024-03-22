use super::vectors::Vec3f;

#[derive(Debug)]
pub enum Attachment {
    Integer(i32),
    Color(Vec3f),
    Texture(String)
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub attachments: Vec<Attachment>
}
