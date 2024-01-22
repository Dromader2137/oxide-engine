use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32
}
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vec2f {
    pub fn new(val: [f32; 2]) -> Vec2f {
        Vec2f { 
            x: val[0], 
            y: val[1], 
        }
    }

    pub fn dot(&mut self, vec: Vec2f) -> f32 {
        self.x * vec.x + self.y * vec.y
    }

    pub fn cross(&mut self, vec: Vec3f) -> f32 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3f {
    pub fn new(val: [f32; 3]) -> Vec3f {
        Vec3f { 
            x: val[0], 
            y: val[1], 
            z: val[2] 
        }
    }

    pub fn dot(&mut self, vec: Vec3f) -> f32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3f) -> Vec3f {
        Vec3f {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x)
        }
    }

    pub fn length_sqr(&mut self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&mut self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&mut self) -> Vec3f {
        let len = self.length();
        Vec3f {
            x: self.x / len, 
            y: self.y / len, 
            z: self.z / len
        }
    }
}
