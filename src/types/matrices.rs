use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::types::vectors::*;

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Matrix4f([[f32; 4]; 4]);

impl Mul for Matrix4f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut output = Matrix4f::indentity();
        for i in (0..4).step_by(1) {
            for j in (0..4).step_by(1) {
                output.0[i][j] = 0.0;
                for k in (0..4).step_by(1) {
                    output.0[i][j] += self.0[k][j] * rhs.0[i][k];
                }
            }
        }
        output
    }
}

impl Matrix4f {
    pub fn indentity() -> Matrix4f {
        Matrix4f([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translation(vec: Vec3f) -> Matrix4f {
        Matrix4f([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [vec.x, vec.y, vec.z, 1.0],
        ])
    }

    pub fn scale(vec: Vec3f) -> Matrix4f {
        Matrix4f([
            [vec.x, 0.0, 0.0, 0.0],
            [0.0, vec.y, 0.0, 0.0],
            [0.0, 0.0, vec.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_x(angle: f32) -> Matrix4f {
        Matrix4f([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin(), 0.0],
            [0.0, -angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_y(angle: f32) -> Matrix4f {
        Matrix4f([
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_z(angle: f32) -> Matrix4f {
        Matrix4f([
            [angle.cos(), angle.sin(), 0.0, 0.0],
            [-angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_yxz(xyz: Vec3f) -> Matrix4f {
        Matrix4f::rotation_y(xyz.y) * Matrix4f::rotation_x(xyz.x) * Matrix4f::rotation_z(xyz.z)
    }

    pub fn rotation_zxy(xyz: Vec3f) -> Matrix4f {
        Matrix4f::rotation_z(xyz.z) * Matrix4f::rotation_x(xyz.x) * Matrix4f::rotation_y(xyz.y)
    }

    pub fn rotation_xzy(xyz: Vec3f) -> Matrix4f {
        Matrix4f::rotation_x(xyz.x) * Matrix4f::rotation_z(xyz.z) * Matrix4f::rotation_y(xyz.y)
    }

    pub fn perspective(fovy: f32, aspect: f32, near: f32) -> Matrix4f {
        let f = 1.0 / (fovy / 2.0).tan();
        Matrix4f([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, 0.0, -1.0],
            [0.0, 0.0, near, 0.0],
        ])
    }

    pub fn look_at(mut eye: Vec3f, mut dir: Vec3f, mut up: Vec3f) -> Matrix4f {
        up.x *= -1.0;
        up.y *= -1.0;
        up.z *= -1.0;
        let mut f = dir.normalize();
        let mut u = f.cross(up.normalize()).normalize();
        let v = u.cross(f);

        Matrix4f([
            [u.x, v.x, f.x, 0.0],
            [u.y, v.y, f.y, 0.0],
            [u.z, v.z, f.z, 0.0],
            [-eye.dot(u), -eye.dot(v), -eye.dot(f), 1.0],
        ])
    }

    pub fn vec_mul(&self, vec: Vec3f) -> Vec3f {
        Vec3f::new([
            vec.x * self.0[0][0] + vec.y * self.0[0][1] + vec.z * self.0[0][2],
            vec.x * self.0[1][0] + vec.y * self.0[1][1] + vec.z * self.0[1][2],
            vec.x * self.0[2][0] + vec.y * self.0[2][1] + vec.z * self.0[2][2],
        ])
    }
}
