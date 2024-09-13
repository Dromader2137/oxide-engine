use std::ops::{Add, AddAssign, Div, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use super::vectors::{Vec3d, Vec3i};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Position {
    pub chunk: Vec3i,
    pub position: Vec3d
}

const CHUNK_SIZE: f64 = 1e12;

impl Position {
    pub fn new(chunk: Vec3i, position: Vec3d) -> Position {
        Position { chunk, position }
    }

    pub fn recalculate_chunk(&mut self) {
        let x_offset = (self.position.x / CHUNK_SIZE) as i64; 
        let y_offset = (self.position.y / CHUNK_SIZE) as i64; 
        let z_offset = (self.position.z / CHUNK_SIZE) as i64; 
        self.position.x -= x_offset as f64 * CHUNK_SIZE;
        self.position.y -= y_offset as f64 * CHUNK_SIZE;
        self.position.z -= z_offset as f64 * CHUNK_SIZE;
        self.chunk += Vec3i::new([x_offset, y_offset, z_offset]);
    }

    pub fn length(&self) -> f64 {
        let vec: Vec3d = self.position + self.chunk.into();
        vec.length()
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            chunk: Vec3i::new([0, 0, 0]),
            position: Vec3d::new([0.0, 0.0, 0.0])
        }
    }
}

impl Add for Position {
    type Output = Position;
    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = Position {
            chunk: self.chunk + rhs.chunk,
            position: self.position + rhs.position
        };
        ret.recalculate_chunk();
        ret
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.chunk += rhs.chunk;
        self.position += rhs.position;
        self.recalculate_chunk();
    }
}

impl Sub for Position {
    type Output = Position;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = Position {
            chunk: self.chunk - rhs.chunk,
            position: self.position - rhs.position
        };
        ret.recalculate_chunk();
        ret
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        self.chunk -= rhs.chunk;
        self.position -= rhs.position;
        self.recalculate_chunk();
    }
}

impl Div<f64> for Position {
    type Output = Position;
    fn div(self, rhs: f64) -> Self::Output {
        let vec: Vec3d = self.into();
        let mut pos: Position = (vec / rhs).into();
        pos.recalculate_chunk();
        pos
    }
}

impl From<Vec3d> for Position {
    fn from(value: Vec3d) -> Self {
        let mut vec = Position { chunk: Vec3i::new([0, 0, 0]), position: value };
        vec.recalculate_chunk();
        vec
    }
}
