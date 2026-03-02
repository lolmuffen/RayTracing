use crate::vector::Vec3;
use crate::material::Material;
use crate::utils::{get_GLOBAL};
use crate::ray::Ray;
use crate::shape::Shape;
use crate::intersection::{Hit, Intersection};

pub struct Sphere {
    pub ID: u32,
    pub position: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(position: Vec3, radius: f32, material: Material) -> Sphere {
        let sphere = Sphere {
            ID: get_GLOBAL().next_object_id(), 
            position, 
            radius, 
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
                let hit_point = ray.origin + ray.direction * t;

                let normal = (hit_point - self.position).normalize();
                let hit = Hit::new(t, hit_point, normal);

                Intersection::new(true, Some(hit), Some(self.ID))
            } else {
                Intersection::new(false, None, None)
            }
        }
    }

    fn get_material(&self) -> &Material {
        &self.material
    }
}