use bytemuck::{Pod, Zeroable};

use crate::types::vectors::*; 

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Matrix4f([[f32; 4]; 4]);

impl Matrix4f {
    pub fn indentity() -> Matrix4f {
        Matrix4f([[1.0, 0.0, 0.0, 0.0],
                  [0.0, 1.0, 0.0, 0.0],
                  [0.0, 0.0, 1.0, 0.0],
                  [0.0, 0.0, 0.0, 1.0]])
    } 
    
    pub fn translation(vec: Vec3f) -> Matrix4f {
        Matrix4f([[1.0, 0.0, 0.0, 0.0],
                  [0.0, 1.0, 0.0, 0.0],
                  [0.0, 0.0, 1.0, 0.0],
                  [-vec.x, vec.y, vec.z, 1.0]])
    }

    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Matrix4f {
        let f = 1.0 / (fovy / 2.0).tan();
        let a = (far + near) / (near - far);
        let b = (2.0 * far * near) / (near - far);
        Matrix4f([[f / aspect, 0.0, 0.0, 0.0],
                  [0.0, f, 0.0, 0.0],
                  [0.0, 0.0, a, -1.0],
                  [0.0, 0.0, b, 0.0]])
    }

    pub fn look_at(mut eye: Vec3f, mut dir: Vec3f, mut up: Vec3f) -> Matrix4f {
        let mut f = dir.normalize();
        let mut s = f.cross(up.normalize()).normalize();
        let u = s.cross(f);

        Matrix4f([[s.x, s.y, s.z, -eye.dot(s)],
                  [u.x, u.y, u.z, -eye.dot(u)],
                  [-f.x, -f.y, -f.z, eye.dot(f)],
                  [0.0, 0.0, 0.0, 1.0]])
    }
}
