use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::types::{matrices::Matrix4f, vectors::Vec3f};

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

impl Quat {
    pub fn new(val: [f32; 4]) -> Quat {
        Quat {
            x: val[0],
            y: val[1],
            z: val[2],
            w: val[3]
        }
    }
    
    pub fn new_sl(val: [f32; 4]) -> Quat {
        Quat {
            x: val[3],
            y: val[0],
            z: val[1],
            w: val[2]
        }
    }

    pub fn inv(&self) -> Quat {
        Quat::new([
            self.x,
            -self.y,
            -self.z,
            -self.w
        ])
    }

    pub fn to_matrix(&self) -> Matrix4f {
        let q = self.normalize();
        let qr = q.x;
        let qi = q.y;
        let qj = q.z;
        let qk = q.w;

        Matrix4f([
            [1.0-2.0*(qj*qj + qk*qk), 2.0*(qi*qk + qj*qr),     2.0*(qi*qj - qk*qr),     0.0],
            [2.0*(qi*qk - qj*qr),     1.0-2.0*(qi*qi + qj*qj), 2.0*(qj*qk + qi*qr),     0.0],
            [2.0*(qi*qj + qk*qr),     2.0*(qj*qk - qi*qr),     1.0-2.0*(qi*qi + qk*qk), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ])
    }

    pub fn from_euler(e: Vec3f) -> Quat {
        let cu = (e.x / 2.0).cos();
        let cw = (e.y / 2.0).cos();
        let cv = (e.z / 2.0).cos();
        
        let su = (e.x / 2.0).sin();
        let sw = (e.y / 2.0).sin();
        let sv = (e.z / 2.0).sin();

        Quat {
            x: cu*cv*cw + su*sv*sw,
            y: su*cv*cw - cu*sv*sw,
            z: cu*sv*cw + su*cv*sw,
            w: cu*cv*sw - su*sv*cw
        }
    }

    pub fn length_sqr(&self) -> f32 {
        self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
    }

    pub fn length(&self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&self) -> Quat {
        let len = self.length();
        Quat::new([self.x / len, self.y / len, self.z / len, self.w / len])
    }
}


impl Mul for Quat {
    type Output = Quat;
    fn mul(self, rhs: Self) -> Self::Output {
        Quat::new([
            self.x*rhs.x - self.y*rhs.y - self.z*rhs.z - self.w*rhs.w,
            self.x*rhs.y + self.y*rhs.x - self.z*rhs.w + self.w*rhs.z,
            self.x*rhs.z + self.y*rhs.w + self.z*rhs.x - self.w*rhs.y,
            self.x*rhs.w - self.y*rhs.z + self.z*rhs.y + self.w*rhs.x,
        ])
    }
}
