use crate::material::Default;
use crate::vector::Vec3;
use crate::sphere::Sphere;
use crate::ray::Ray;
use crate::utils::{get_GLOBAL, random_double};
use std::f32::consts::PI;

pub struct SphereLight {
    ID: u32,
    structure: Sphere,
    intensity: f32,
    color: Vec3,
}

impl SphereLight {
    pub fn new(position: Vec3, radius: f32, color: Vec3, intensity: f32) -> Self {
       SphereLight { ID: get_GLOBAL().next_light_id(), structure: Sphere { ID: 0, position, radius, color, material: Box::new(Default::new()) }, intensity, color }
    }

    pub fn sample_sphere_light(&self, pos: Vec3) -> Ray {
        let u1 = random_double();
        let u2 = random_double();

        let cos_theta = 1.0 - 2.0 * u1;
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * PI * u2;

        let x = sin_theta * phi.cos();
        let y = sin_theta * phi.sin();
        let z = cos_theta;

        Ray::new(pos, (self.structure.position + self.structure.radius * Vec3::new(x, y, z)) - pos, self.color * self.intensity)
    }
}

impl Light for SphereLight {
    fn sample_light(&self, pos: Vec3) -> Ray {
        self.sample_sphere_light(pos)
    }
    fn get_id(&self) -> u32 {
        self.ID
    }
    fn get_position(&self) -> Vec3 {
        return self.structure.position;
    }
    fn get_color(&self) -> Vec3 {
        return self.color;
    }
    fn get_intensity(&self) -> f32 {
        return self.intensity;
    }
}

pub trait Light {
    fn sample_light(&self, pos: Vec3) -> Ray;
    fn get_id(&self) -> u32;
    fn get_position(&self) -> Vec3;
    fn get_intensity(&self) -> f32;
    fn get_color(&self) -> Vec3;
}