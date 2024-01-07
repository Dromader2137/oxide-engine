#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Matrix4f([[f32; 4]; 4]);

impl Matrix4f {
    pub fn indentity() -> Matrix4f {
        Matrix4f([[1.0, 0.0, 0.0, 0.0],
                  [0.0, 1.0, 0.0, 0.0],
                  [0.0, 0.0, 1.0, 0.0],
                  [0.0, 0.0, 0.0, 1.0]])
    } 
}
