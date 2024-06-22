use std::ops::Mul;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize)]
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
