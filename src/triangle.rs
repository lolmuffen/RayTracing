use crate::material::{Material};
use crate::ray::Ray;
use crate::shape::Shape;
use crate::utils::get_GLOBAL;
use crate::vector::Vec3;
use crate::intersection::{Hit, Intersection};

pub struct Triangle {
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    normal: Vec3,
    material: Material,
    id: u32,
}

impl Triangle {
    pub fn new(p1: Vec3, p2: Vec3, p3: Vec3, mat: Material) -> Self {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e1.cross(e2).normalize();
        return Triangle { p1, p2, p3, normal, material: mat, id: get_GLOBAL().next_object_id() }
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray) -> Intersection {
        //This function uses moller_trumbore_intersection
        //I dont fully understand this math

        let e1 = self.p2 - self.p1;
        let e2 = self.p3 - self.p1;

        let ray_cross_e2 = ray.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return Intersection { hit: false, hitdata: None, object_id: None }; // This ray is parallel to this triangle.
        }

        let inv_det = 1.0 / det;
        let s = ray.origin - self.p1;
        let u = inv_det * s.dot(ray_cross_e2);
        if u < 0.0 || u > 1.0 {
            return Intersection { hit: false, hitdata: None, object_id: None };
        }

        let s_cross_e1 = s.cross(e1);
        let v = inv_det * ray.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return Intersection { hit: false, hitdata: None, object_id: None };
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t > f32::EPSILON { // ray intersection
            let intersection_point = ray.origin + ray.direction * t;
            return Intersection { hit: true, hitdata: Some(Hit::new(t, intersection_point, self.normal)), object_id: Some(self.id) };
        }
        else { // This means that there is a line intersection but not a ray intersection.
            return Intersection { hit: false, hitdata: None, object_id: None };
        }
    }

    fn get_id(&self) -> u32 {
        return self.id
    }

    fn get_material(&self) -> &Material {
        return &self.material;
    }

    fn get_max_bounds(&self) -> Vec3 {
        let max_x = self.p1.x.max(self.p2.x).max(self.p3.x);
        let max_y = self.p1.y.max(self.p2.y).max(self.p3.y);
        let max_z = self.p1.z.max(self.p2.z).max(self.p3.z);
        return Vec3::new(max_x, max_y, max_z)
    }

    fn get_min_bounds(&self) -> Vec3 {
        let min_x = self.p1.x.min(self.p2.x).min(self.p3.x);
        let min_y = self.p1.y.min(self.p2.y).min(self.p3.y);
        let min_z = self.p1.z.min(self.p2.z).min(self.p3.z);
        return Vec3::new(min_x, min_y, min_z)
    }
}