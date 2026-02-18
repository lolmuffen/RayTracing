use std::f32::consts::PI;
use crate::material::Emissive;
use crate::ray::Ray;
use crate::render::Color;
use crate::sphere::Sphere;
use crate::utils::random_double;
use crate::vector::Vec3;

pub struct Light {
    pub sphere: Sphere,
}

impl Light {
    pub fn new(position: Vec3, radius: f32, color: Color, intensity: f32) -> Self {
        Light {sphere: Sphere::new(position.x, position.y, position.z, radius, Box::new(Emissive::new(color, intensity)))}
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

        Ray::new(pos, ((self.sphere.center + self.sphere.radius * Vec3::new(x, y, z)) - pos))
    }
}