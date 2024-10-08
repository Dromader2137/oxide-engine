use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::types::vectors::*;

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Matrix4f(pub [[f32; 4]; 4]);

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Matrix4d(pub [[f64; 4]; 4]);

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

    pub fn rotation_zyx(xyz: Vec3f) -> Matrix4f {
        Matrix4f::rotation_z(xyz.z) * Matrix4f::rotation_y(xyz.y) * Matrix4f::rotation_x(xyz.x)
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

    pub fn look_at(eye: Vec3f, dir: Vec3f, mut up: Vec3f) -> Matrix4f {
        up.x *= -1.0;
        up.y *= -1.0;
        up.z *= -1.0;
        let f = dir.normalize();
        let u = f.cross(up.normalize()).normalize();
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
            vec.x * self.0[0][0] + vec.y * self.0[1][0] + vec.z * self.0[2][0],
            vec.x * self.0[0][1] + vec.y * self.0[1][1] + vec.z * self.0[2][1],
            vec.x * self.0[0][2] + vec.y * self.0[1][2] + vec.z * self.0[2][2],
        ])
    }

    pub fn vec_mul_inv(&self, vec: Vec3f) -> Vec3f {
        Vec3f::new([
            vec.x * self.0[0][0] + vec.y * self.0[0][1] + vec.z * self.0[0][2],
            vec.x * self.0[1][0] + vec.y * self.0[1][1] + vec.z * self.0[1][2],
            vec.x * self.0[2][0] + vec.y * self.0[2][1] + vec.z * self.0[2][2],
        ])
    }
}

impl Mul for Matrix4d {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut output = Matrix4d::indentity();
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

impl Matrix4d {
    pub fn indentity() -> Matrix4d {
        Matrix4d([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translation(vec: Vec3d) -> Matrix4d {
        Matrix4d([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [vec.x, vec.y, vec.z, 1.0],
        ])
    }

    pub fn scale(vec: Vec3d) -> Matrix4d {
        Matrix4d([
            [vec.x, 0.0, 0.0, 0.0],
            [0.0, vec.y, 0.0, 0.0],
            [0.0, 0.0, vec.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_x(angle: f64) -> Matrix4d {
        Matrix4d([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), angle.sin(), 0.0],
            [0.0, -angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_y(angle: f64) -> Matrix4d {
        Matrix4d([
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_z(angle: f64) -> Matrix4d {
        Matrix4d([
            [angle.cos(), angle.sin(), 0.0, 0.0],
            [-angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_yxz(xyz: Vec3d) -> Matrix4d {
        Matrix4d::rotation_y(xyz.y) * Matrix4d::rotation_x(xyz.x) * Matrix4d::rotation_z(xyz.z)
    }

    pub fn rotation_zxy(xyz: Vec3d) -> Matrix4d {
        Matrix4d::rotation_z(xyz.z) * Matrix4d::rotation_x(xyz.x) * Matrix4d::rotation_y(xyz.y)
    }

    pub fn rotation_xzy(xyz: Vec3d) -> Matrix4d {
        Matrix4d::rotation_x(xyz.x) * Matrix4d::rotation_z(xyz.z) * Matrix4d::rotation_y(xyz.y)
    }

    pub fn rotation_zyx(xyz: Vec3d) -> Matrix4d {
        Matrix4d::rotation_z(xyz.z) * Matrix4d::rotation_y(xyz.y) * Matrix4d::rotation_x(xyz.x)
    }

    pub fn perspective(fovy: f64, aspect: f64, near: f64) -> Matrix4d {
        let f = 1.0 / (fovy / 2.0).tan();
        Matrix4d([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, 0.0, -1.0],
            [0.0, 0.0, near, 0.0],
        ])
    }

    pub fn look_at(eye: Vec3d, dir: Vec3d, mut up: Vec3d) -> Matrix4d {
        up.x *= -1.0;
        up.y *= -1.0;
        up.z *= -1.0;
        let f = dir.normalize();
        let u = f.cross(up.normalize()).normalize();
        let v = u.cross(f);

        Matrix4d([
            [u.x, v.x, f.x, 0.0],
            [u.y, v.y, f.y, 0.0],
            [u.z, v.z, f.z, 0.0],
            [-eye.dot(u), -eye.dot(v), -eye.dot(f), 1.0],
        ])
    }

    pub fn vec_mul(&self, vec: Vec3d) -> Vec3d {
        Vec3d::new([
            vec.x * self.0[0][0] + vec.y * self.0[1][0] + vec.z * self.0[2][0],
            vec.x * self.0[0][1] + vec.y * self.0[1][1] + vec.z * self.0[2][1],
            vec.x * self.0[0][2] + vec.y * self.0[1][2] + vec.z * self.0[2][2],
        ])
    }

    pub fn vec_mul_inv(&self, vec: Vec3d) -> Vec3d {
        Vec3d::new([
            vec.x * self.0[0][0] + vec.y * self.0[0][1] + vec.z * self.0[0][2],
            vec.x * self.0[1][0] + vec.y * self.0[1][1] + vec.z * self.0[1][2],
            vec.x * self.0[2][0] + vec.y * self.0[2][1] + vec.z * self.0[2][2],
        ])
    }
}
