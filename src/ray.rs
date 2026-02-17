

use crate::vector::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, color: Vec3) -> Ray {
        Ray { origin, direction, color }
    }
}