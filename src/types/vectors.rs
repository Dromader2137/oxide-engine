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

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64
}
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Vec2f {
    pub fn new(val: [f32; 2]) -> Vec2f {
        Vec2f { 
            x: val[0], 
            y: val[1], 
        }
    }
    
    pub fn from_vec2d(val: Vec2d) -> Vec2f {
        Vec2f { 
            x: val.x as f32, 
            y: val.y as f32, 
        }
    }
    
    pub fn to_vec2d(&self) -> Vec2d {
        Vec2d { 
            x: self.x as f64, 
            y: self.y as f64, 
        }
    }

    pub fn dot(&mut self, vec: Vec2f) -> f32 {
        self.x * vec.x + self.y * vec.y
    }

    pub fn cross(&mut self, vec: Vec2f) -> f32 {
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
    
    pub fn from_vec3d(val: Vec3d) -> Vec3f {
        Vec3f { 
            x: val.x as f32, 
            y: val.y as f32, 
            z: val.z as f32, 
        }
    }
    
    pub fn to_vec3d(&self) -> Vec3d {
        Vec3d { 
            x: self.x as f64, 
            y: self.y as f64, 
            z: self.z as f64, 
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
        }
    }

    pub fn dot(&mut self, vec: Vec2d) -> f64 {
        self.x * vec.x + self.y * vec.y
    }

    pub fn cross(&mut self, vec: Vec2d) -> f64 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3d {
    pub fn new(val: [f64; 3]) -> Vec3d {
        Vec3d { 
            x: val[0], 
            y: val[1], 
            z: val[2] 
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
        }
    }

    pub fn dot(&mut self, vec: Vec3d) -> f64 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3d) -> Vec3d {
        Vec3d {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x)
        }
    }

    pub fn length_sqr(&mut self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&mut self) -> f64 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&mut self) -> Vec3d {
        let len = self.length();
        Vec3d {
            x: self.x / len, 
            y: self.y / len, 
            z: self.z / len
        }
    }
}
