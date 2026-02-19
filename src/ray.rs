

use crate::vector::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub light: Vec3,
    pub inv_dir: Vec3, // Precompute inverse direction for faster intersection tests
    
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, color: Vec3) -> Ray {
        let inv_dir = Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);
        Ray { origin, direction, color, light: Vec3::new(1.0, 1.0, 1.0), inv_dir }

    }

}