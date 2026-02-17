mod vector;
mod utils;
mod ray;
mod sphere;
mod material;
mod intersection;
mod shape;
mod BVH;
mod camera;


use crate::vector::Vec3;
use crate::camera::Camera;

fn main() {
    let cam = Camera::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), (800, 600), 60);
    cam.render();
}

