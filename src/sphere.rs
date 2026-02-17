use crate::vector::Vec3;
use crate::material::Material;
use crate::utils::GLOBAL;
use crate::ray::Ray;
use crate::shape::Shape;
use crate::intersection::{Hit, Intersection};

pub struct Sphere {
    pub ID: u32,
    pub position: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub material: Material,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32, color: Vec3, material: Material) -> Sphere {
        let sphere = Sphere {
            ID: GLOBAL.next_object_id(), 
            position, 
            radius, 
            color, 
            material 
        };
        sphere
    }
}


impl Shape for Sphere {

    fn get_id(&self) -> u32 {
        return self.ID;
    }

    fn get_max_bounds(&self) -> Vec3 {
        Vec3::new(
            self.position.x + self.radius,
            self.position.y + self.radius,
            self.position.z + self.radius,
        )
    }

    fn get_min_bounds(&self) -> Vec3 {
        Vec3::new(
            self.position.x - self.radius,
            self.position.y - self.radius,
            self.position.z - self.radius,
        )
    }

    fn intersect(&self, ray: &Ray) -> Intersection {
        let oc = ray.origin - self.position;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            Intersection::new(false, None, None)
        } else {
            let sqrt_disc = discriminant.sqrt();
            let t1 = (-b - sqrt_disc) / (2.0 * a);
            let t2 = (-b + sqrt_disc) / (2.0 * a);
            let t = if t1 > 0.001 { t1 } else { t2 };


            if t > 0.001 {
                let hit_point_1 = ray.origin + ray.direction * t1;
                let hit_point_2 = ray.origin + ray.direction * t2;
                let normal_1 = (hit_point_1 - self.position).normalize();
                let normal_2 = (hit_point_2 - self.position).normalize();
                let hit_1 = Hit::new(t1, hit_point_1, normal_1);
                let hit_2 = Hit::new(t2, hit_point_2, normal_2);
                Intersection::new(true, Some(vec![hit_1, hit_2]), Some(self.ID))
            } else {
                Intersection::new(false, None, None)
            }
        }
    }
}