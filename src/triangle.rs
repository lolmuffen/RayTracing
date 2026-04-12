use crate::material::{Material};
use crate::vector::Vec3;
use crate::ray::Ray;
use crate::intersection::{Hit, Intersection};

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

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        // Möller-Trumbore intersection algorithm
        let e1 = self.p2 - self.p1;
        let e2 = self.p3 - self.p1;

        let ray_cross_e2 = ray.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.p1;
        let u = inv_det * s.dot(ray_cross_e2);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let s_cross_e1 = s.cross(e1);
        let v = inv_det * ray.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t > f32::EPSILON { // ray intersection
            let intersection_point = ray.origin + ray.direction * t;
            let front_face = ray.direction.dot(self.normal) < 0.0;
            let normal = if front_face { self.normal } else { -self.normal };
            let hit = Hit::new_with_material(t, intersection_point, normal, front_face, self.material.clone());
            Some(Intersection::new(true, Some(hit), Some(self.id)))
        } else {
            None
        }
    }
}