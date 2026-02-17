use crate::ray::Ray;
use crate::intersection::Intersection;


pub trait Shape {
    /// Check if a ray intersects this shape
    /// Returns Some(Intersection) if hit, None if miss
    fn intersect(&self, ray: &Ray) -> Intersection;
}
