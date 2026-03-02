use crate::material::{Material};
use crate::utils::get_GLOBAL;
use crate::vector::Vec3;

pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub id: u32,
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3, mat: Material) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e1.cross(e2).normalize();
        return Triangle { p1, p2, p3, normal, material: mat, id: get_GLOBAL().next_object_id() }
    }
}
