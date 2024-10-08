use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use super::{position::Position, quaternion::Quat};

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
    _align: i64
}
#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _align: i32
}

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Pod, Zeroable, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3i {
    pub x: i64,
    pub y: i64,
    pub z: i64
}

impl Add for Vec2f {
    type Output = Vec2f;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2f::new([self.x + rhs.x, self.y + rhs.y])
    }
}

impl AddAssign for Vec2f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3f {
    type Output = Vec3f;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3f::new([self.x + rhs.x, self.y + rhs.y, self.z + rhs.z])
    }
}

impl AddAssign for Vec3f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vec2d {
    type Output = Vec2d;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2d::new([self.x + rhs.x, self.y + rhs.y])
    }
}

impl AddAssign for Vec2d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3d {
    type Output = Vec3d;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3d::new([self.x + rhs.x, self.y + rhs.y, self.z + rhs.z])
    }
}

impl AddAssign for Vec3d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vec3i {
    type Output = Vec3i;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3i::new([self.x + rhs.x, self.y + rhs.y, self.z + rhs.z])
    }
}

impl AddAssign for Vec3i {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Div for Vec2f {
    type Output = Vec2f;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2f::new([self.x / rhs.x, self.y / rhs.y])
    }
}

impl Div<f32> for Vec2f {
    type Output = Vec2f;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2f::new([self.x / rhs, self.y / rhs])
    }
}

impl DivAssign for Vec2f {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f32> for Vec2f {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3f {
    type Output = Vec3f;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3f::new([self.x / rhs.x, self.y / rhs.y, self.z / rhs.z])
    }
}

impl Div<f32> for Vec3f {
    type Output = Vec3f;
    fn div(self, rhs: f32) -> Self::Output {
        Vec3f::new([self.x / rhs, self.y / rhs, self.z / rhs])
    }
}

impl DivAssign for Vec3f {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f32> for Vec3f {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Div for Vec2d {
    type Output = Vec2d;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2d::new([self.x / rhs.x, self.y / rhs.y])
    }
}

impl Div<f64> for Vec2d {
    type Output = Vec2d;
    fn div(self, rhs: f64) -> Self::Output {
        Vec2d::new([self.x / rhs, self.y / rhs])
    }
}

impl DivAssign for Vec2d {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f64> for Vec2d {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3d {
    type Output = Vec3d;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3d::new([self.x / rhs.x, self.y / rhs.y, self.z / rhs.z])
    }
}

impl Div<f64> for Vec3d {
    type Output = Vec3d;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3d::new([self.x / rhs, self.y / rhs, self.z / rhs])
    }
}

impl DivAssign for Vec3d {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f64> for Vec3d {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Div for Vec3i {
    type Output = Vec3i;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3i::new([self.x / rhs.x, self.y / rhs.y, self.z / rhs.z])
    }
}

impl Div<i64> for Vec3i {
    type Output = Vec3i;
    fn div(self, rhs: i64) -> Self::Output {
        Vec3i::new([self.x / rhs, self.y / rhs, self.z / rhs])
    }
}

impl DivAssign for Vec3i {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<i64> for Vec3i {
    fn div_assign(&mut self, rhs: i64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Mul<f32> for Vec2f {
    type Output = Vec2f;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2f::new([self.x * rhs, self.y * rhs])
    }
}

impl MulAssign for Vec2f {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f32> for Vec2f {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3f::new([self.x * rhs.x, self.y * rhs.y, self.z * rhs.z])
    }
}

impl Mul<f32> for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3f::new([self.x * rhs, self.y * rhs, self.z * rhs])
    }
}

impl MulAssign for Vec3f {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f32> for Vec3f {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec2d::new([self.x * rhs.x, self.y * rhs.y])
    }
}

impl Mul<f64> for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec2d::new([self.x * rhs, self.y * rhs])
    }
}

impl MulAssign for Vec2d {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f64> for Vec2d {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3d {
    type Output = Vec3d;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3d::new([self.x * rhs.x, self.y * rhs.y, self.z * rhs.z])
    }
}

impl Mul<f64> for Vec3d {
    type Output = Vec3d;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3d::new([self.x * rhs, self.y * rhs, self.z * rhs])
    }
}

impl MulAssign for Vec3d {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f64> for Vec3d {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul for Vec3i {
    type Output = Vec3i;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3i::new([self.x * rhs.x, self.y * rhs.y, self.z * rhs.z])
    }
}

impl Mul<i64> for Vec3i {
    type Output = Vec3i;
    fn mul(self, rhs: i64) -> Self::Output {
        Vec3i::new([self.x * rhs, self.y * rhs, self.z * rhs])
    }
}

impl MulAssign for Vec3i {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<i64> for Vec3i {
    fn mul_assign(&mut self, rhs: i64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Sub for Vec2f {
    type Output = Vec2f;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2f::new([self.x - rhs.x, self.y - rhs.y])
    }
}

impl SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Vec3f {
    type Output = Vec3f;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3f::new([self.x - rhs.x, self.y - rhs.y, self.z - rhs.z])
    }
}

impl SubAssign for Vec3f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for Vec2d {
    type Output = Vec2d;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2d::new([self.x - rhs.x, self.y - rhs.y])
    }
}

impl SubAssign for Vec2d {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Vec3d {
    type Output = Vec3d;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3d::new([self.x - rhs.x, self.y - rhs.y, self.z - rhs.z])
    }
}

impl SubAssign for Vec3d {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for Vec3i {
    type Output = Vec3i;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3i::new([self.x - rhs.x, self.y - rhs.y, self.z - rhs.z])
    }
}

impl SubAssign for Vec3i {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Vec2f {
    pub fn new(val: [f32; 2]) -> Vec2f {
        Vec2f {
            x: val[0],
            y: val[1],
            _align: 0
        }
    }

    pub fn from_vec2d(val: Vec2d) -> Vec2f {
        Vec2f {
            x: val.x as f32,
            y: val.y as f32,
            _align: 0
        }
    }

    pub fn to_vec2d(&self) -> Vec2d {
        Vec2d {
            x: self.x as f64,
            y: self.y as f64,
        }
    }

    pub fn dot(&self, vec: Vec2f) -> f32 {
        self.x * vec.x + self.y * vec.y
    }

    pub fn cross(&self, vec: Vec2f) -> f32 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3f {
    pub fn new(val: [f32; 3]) -> Vec3f {
        Vec3f {
            x: val[0],
            y: val[1],
            z: val[2],
            _align: 0
        }
    }

    pub fn from_vec3d(val: Vec3d) -> Vec3f {
        Vec3f {
            x: val.x as f32,
            y: val.y as f32,
            z: val.z as f32,
            _align: 0
        }
    }

    pub fn to_vec3d(&self) -> Vec3d {
        Vec3d {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }

    pub fn dot(&self, vec: Vec3f) -> f32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&self, vec: Vec3f) -> Vec3f {
        Vec3f {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
            _align: 0
        }
    }

    pub fn length_sqr(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&self) -> Vec3f {
        let len = self.length();
        Vec3f {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            _align: 0
        }
    }
}

impl Vec2d {
    pub fn new(val: [f64; 2]) -> Vec2d {
        Vec2d {
            x: val[0],
            y: val[1],
        }
    }

    pub fn from_vec2f(val: Vec2f) -> Vec2d {
        Vec2d {
            x: val.x as f64,
            y: val.y as f64,
        }
    }

    pub fn to_vec2f(&self) -> Vec2f {
        Vec2f {
            x: self.x as f32,
            y: self.y as f32,
            _align: 0
        }
    }

    pub fn dot(&self, vec: Vec2d) -> f64 {
        self.x * vec.x + self.y * vec.y
    }

    pub fn cross(&self, vec: Vec2d) -> f64 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3d {
    pub fn new(val: [f64; 3]) -> Vec3d {
        Vec3d {
            x: val[0],
            y: val[1],
            z: val[2],
        }
    }

    pub fn from_vec3f(val: Vec3f) -> Vec3d {
        Vec3d {
            x: val.x as f64,
            y: val.y as f64,
            z: val.z as f64,
        }
    }

    pub fn to_vec3f(&self) -> Vec3f {
        Vec3f {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
            _align: 0
        }
    }

    pub fn dot(&self, vec: Vec3d) -> f64 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&self, vec: Vec3d) -> Vec3d {
        Vec3d {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
        }
    }

    pub fn length_sqr(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&self) -> Vec3d {
        let len = self.length();
        Vec3d {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

impl Vec3i {
    pub fn new(val: [i64; 3]) -> Vec3i {
        Vec3i {
            x: val[0],
            y: val[1],
            z: val[2]
        }
    }
    
    pub fn length_sqr(&self) -> i64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    
    pub fn length(&self) -> f64 {
        (self.length_sqr() as f64).sqrt()
    }
}

impl From<Vec4f> for Vec3f {
    fn from(value: Vec4f) -> Self {
        Vec3f::new([value.x, value.y, value.z])
    }
}

impl Vec4f {
    pub fn new(val: [f32; 4]) -> Vec4f {
        Vec4f {
            x: val[0],
            y: val[1],
            z: val[2],
            w: val[3]
        }
    }
    
    pub fn length_sqr_xyz(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    
    pub fn length_xyz(&self) -> f32 {
        self.length_sqr_xyz().sqrt()
    }
    
    pub fn normalize_xyz(&self) -> Vec4f {
        let len = self.length_xyz();
        Vec4f {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w
        }
    }
}

impl Mul<Quat> for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: Quat) -> Self::Output {
        let p = Quat::new([0.0, self.x, self.y, self.z]);
        let pp = rhs.inv() * p * rhs;
        Vec3f::new([pp.x, pp.y, pp.z])
    }
}

impl From<Vec3i> for Vec3d {
    fn from(value: Vec3i) -> Self {
        Vec3d::new([value.x as f64, value.y as f64, value.z as f64])
    }
}

impl From<Position> for Vec3d {
    fn from(value: Position) -> Self {
        value.position + value.chunk.into()
    }
}

impl From<Position> for Vec3f {
    fn from(value: Position) -> Self {
        (value.position + value.chunk.into()).to_vec3f()
    }
}
