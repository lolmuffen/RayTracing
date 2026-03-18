use crate::material::{Material};
use crate::vector::Vec3;

#[derive(Clone)]
pub struct Triangle {
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
    pub normal: Vec3,
    pub material: Material,
    /// 0 = internal BVH triangle (not a registered scene object).
    /// Set to a real global ID only when the triangle is used as a
    /// standalone Shape::Triangle in the scene.
    pub id: u32,
}

impl Triangle {
    /// Internal BVH triangle — id is 0 (not a scene object).
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3, mat: Material) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e1.cross(e2).normalize();
        Triangle { p1, p2, p3, normal, material: mat, id: 0 }
    }

    /// Internal BVH triangle with an explicit normal — id is 0.
    pub fn new_with_normal(p1: Vec3, p2: Vec3, p3: Vec3, normal: Vec3, mat: Material) -> Self {
        Triangle { p1, p2, p3, normal, material: mat, id: 0 }
    }

    /// Standalone scene triangle that needs its own registered ID.
    pub fn new_scene_object(p1: Vec3, p2: Vec3, p3: Vec3, mat: Material, id: u32) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e1.cross(e2).normalize();
        Triangle { p1, p2, p3, normal, material: mat, id }
    }
}