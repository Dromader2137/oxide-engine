use std::ops::{Add, Mul};

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
            w: val[0],
            x: val[1],
            y: val[2],
            z: val[3]
        }
    }
    
    pub fn new_sl(val: [f32; 4]) -> Quat {
        Quat {
            w: val[3],
            x: val[0],
            y: val[1],
            z: val[2]
        }
    }

    pub fn inv(&self) -> Quat {
        Quat::new([
            self.w,
            -self.x,
            -self.y,
            -self.z,
        ])
    }

    pub fn to_matrix(&self) -> Matrix4f {
        let q = self.normalize();
        let qr = q.w;
        let qi = q.x;
        let qj = q.z;
        let qk = q.y;

        Matrix4f([
            [1.0-2.0*(qj*qj + qk*qk), 2.0*(qi*qk + qj*qr),     2.0*(qi*qj - qk*qr),     0.0],
            [2.0*(qi*qk - qj*qr),     1.0-2.0*(qi*qi + qj*qj), 2.0*(qj*qk + qi*qr),     0.0],
            [2.0*(qi*qj + qk*qr),     2.0*(qj*qk - qi*qr),     1.0-2.0*(qi*qi + qk*qk), 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ])
    }

    pub fn from_euler(e: Vec3f) -> Quat {
        let cr = (e.x / 2.0).cos();
        let cy = (e.y / 2.0).cos();
        let cp = (e.z / 2.0).cos();
        
        let sr = (e.x / 2.0).sin();
        let sy = (e.y / 2.0).sin();
        let sp = (e.z / 2.0).sin();

        Quat {
            w: cr*cy*cp + sr*sy*sp,
            x: sr*cy*cp - cr*sy*sp,
            y: cr*sy*cp - sr*cy*sp,
            z: cr*cy*sp + sr*sy*cp,
        }.normalize()
    }

    pub fn length_sqr(&self) -> f32 {
        self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w
    }

    pub fn length(&self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&self) -> Quat {
        let len = self.length();
        Quat::new([self.w / len, self.x / len, self.y / len, self.z / len])
    }
}

impl Add for Quat {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Quat::new([self.w + rhs.w, self.x + rhs.x, self.y + rhs.y, self.z + rhs.z])
    }
}

impl Mul for Quat {
    type Output = Quat;
    fn mul(self, rhs: Self) -> Self::Output {
        Quat::new([
            self.w*rhs.w - self.x*rhs.x - self.z*rhs.z - self.y*rhs.y,
            self.w*rhs.x + self.x*rhs.w + self.z*rhs.y - self.y*rhs.z,
            self.w*rhs.y + self.x*rhs.z - self.z*rhs.x + self.y*rhs.w,
            self.w*rhs.z - self.x*rhs.y + self.z*rhs.w + self.y*rhs.x,
        ])
    }
}

impl Mul<Vec3f> for Quat {
    type Output = Vec3f;
    fn mul(self, rhs: Vec3f) -> Vec3f {
        let p = Quat::new([0.0, rhs.x, rhs.y, rhs.z]);
        let pp = self * p * self.inv();
        Vec3f::new([pp.x, pp.y, pp.z])
    }
}

impl Mul<f32> for Quat {
    type Output = Quat;
    fn mul(self, rhs: f32) -> Quat {
        Quat::new([self.w * rhs, self.x * rhs, self.y * rhs, self.z * rhs])
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use nalgebra::UnitQuaternion;

    use crate::types::{quaternion::Quat, vectors::Vec3f};

    extern crate nalgebra as na;

    #[test]
    fn test_quat_from_euler_na() {
        let my_quat = Quat::from_euler(Vec3f::new([0.0, 0.4, 0.0]));
        let na_quat = UnitQuaternion::from_euler_angles(0.0, 0.4, 0.0);
       
        assert_eq!([na_quat.i, na_quat.j, na_quat.k, na_quat.w], 
                   [my_quat.x, my_quat.y, my_quat.z, my_quat.w]);
    }

    #[test]
    fn test_quat_from_euler_y() {
        let my_quat = Quat::from_euler(Vec3f::new([0.0, 0.4, 0.0]));
       
        assert_relative_eq!(my_quat.x, 0.0);
        assert_relative_eq!(my_quat.y, 0.199, epsilon=1e-3);
        assert_relative_eq!(my_quat.z, 0.0);
        assert_relative_eq!(my_quat.w, 0.98, epsilon=1e-3);
    }
    
    #[test]
    fn test_quat_from_euler_x() {
        let my_quat = Quat::from_euler(Vec3f::new([0.4, 0.0, 0.0]));
       
        assert_relative_eq!(my_quat.x, 0.199, epsilon=1e-3);
        assert_relative_eq!(my_quat.y, 0.0);
        assert_relative_eq!(my_quat.z, 0.0);
        assert_relative_eq!(my_quat.w, 0.98, epsilon=1e-3);
    }
    
    #[test]
    fn test_quat_from_euler_z() {
        let my_quat = Quat::from_euler(Vec3f::new([0.0, 0.0, 0.4]));
       
        assert_relative_eq!(my_quat.x, 0.0);
        assert_relative_eq!(my_quat.y, 0.0);
        assert_relative_eq!(my_quat.z, 0.199, epsilon=1e-3);
        assert_relative_eq!(my_quat.w, 0.98, epsilon=1e-3);
    }


    #[test]
    fn test_quat_mul_na() {
        let my_quat_1 = Quat::from_euler(Vec3f::new([0.1, 1.6, 0.4]));
        let na_quat_1 = UnitQuaternion::from_euler_angles(0.1, 0.4, 1.6);

        let my_quat_2 = Quat::from_euler(Vec3f::new([-0.3, 0.6, -1.4]));
        let na_quat_2 = UnitQuaternion::from_euler_angles(-0.3, -1.4, 0.6);

        let my_quat = my_quat_1 * my_quat_2;
        let na_quat = na_quat_1 * na_quat_2;

        assert_relative_eq!(my_quat.x, na_quat.i);
        assert_relative_eq!(my_quat.y, na_quat.k);
        assert_relative_eq!(my_quat.z, na_quat.j);
        assert_relative_eq!(my_quat.w, na_quat.w);
    }
    
    #[test]
    fn test_quat_vec_mul() {
        let my_quat = Quat::from_euler(Vec3f::new([0.0, 0.4, 0.0]));
        let my_vec_base = Vec3f::new([0.0, 1.0, 0.0]);
        let my_vec = my_quat * my_vec_base;

        assert_relative_eq!(my_vec.y, 1.0);
        assert_relative_eq!(my_vec.x, 0.0);
        assert_relative_eq!(my_vec.z, 0.0);
    }
}
