use crate::ray::Ray;
use crate::vector::Vec3;
use crate::intersection::Intersection;
use crate::material::Material;

pub trait Shape {
    /// Check if a ray intersects this shape
    /// Returns Some(Intersection) if hit, None if miss
    fn intersect(&self, ray: &Ray) -> Intersection;
    fn get_max_bounds(&self) -> Vec3;
    fn get_min_bounds(&self) -> Vec3;
    fn get_id(&self) -> u32;
    fn get_material(&self) -> &Box<dyn Material + Send + Sync>;
}
