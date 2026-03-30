use crate::vector::Vec3;
use crate::material::Material;

pub struct Sphere {
    pub ID: u32,
    pub position: Vec3,
    pub radius: f32,
    pub material: Material,
}

