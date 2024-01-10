#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct Vec2f(pub [f32; 2]);
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct Vec3f(pub [f32; 3]);

impl Vec3f {
    pub fn x(&mut self) -> &mut f32 { self.0.get_mut(0).unwrap() }
    pub fn y(&mut self) -> &mut f32 { self.0.get_mut(1).unwrap() }
    pub fn z(&mut self) -> &mut f32 { self.0.get_mut(2).unwrap() }

    pub fn dot(&mut self, mut vec: Vec3f) -> f32 {
        *self.x() * *vec.x() + *self.y() * *vec.y() + *self.z() * *vec.z()
    }

    pub fn cross(&mut self, mut vec: Vec3f) -> Vec3f {
        Vec3f(
            [(*self.y() * *vec.z()) - (*self.z() * *vec.y()),
             (*self.z() * *vec.x()) - (*self.x() * *vec.z()),
             (*self.x() * *vec.y()) - (*self.y() * *vec.x())]
            )
    }

    pub fn length_sqr(&mut self) -> f32 {
        *self.x() * *self.x() + *self.y() * *self.y() + *self.z() * *self.z()
    }

    pub fn length(&mut self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalize(&mut self) -> Vec3f {
        let len = self.length();
        Vec3f([*self.x() / len, *self.y() / len, *self.z() / len])
    }
}
