use nalgebra_glm as glm;
use glm::{TVec, TVec1, Vec3};


pub trait Conventer<T: Copy> {
    type Tuple;
    type Array;
    fn tuple(&self) -> Self::Tuple;
    fn array(&self) -> Self::Array;
}

impl Conventer<f32> for Vec3 {
    type Tuple = (f32, f32, f32);
    type Array = [f32; 3];
    #[inline] fn tuple(&self) -> Self::Tuple { (self.x, self.y, self.z) }
    #[inline] fn array(&self) -> Self::Array { [self.x, self.y, self.z] }
}