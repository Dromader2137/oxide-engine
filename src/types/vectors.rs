#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec2f(pub [f32; 2]);
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec3f(pub [f32; 3]);

impl Vec3f {
    pub fn x(&mut self) -> &mut f32 { self.0.get_mut(0).unwrap() }
    pub fn y(&mut self) -> &mut f32 { self.0.get_mut(1).unwrap() }
    pub fn z(&mut self) -> &mut f32 { self.0.get_mut(2).unwrap() }
}
