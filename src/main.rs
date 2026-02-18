mod vector;
mod utils;
mod ray;
mod sphere;
mod material;
mod intersection;
mod shape;
mod BVH;
mod camera;
mod light;


use crate::vector::Vec3;
use crate::camera::Camera;
use crate::utils::{GLOBAL, Global};

fn main() {
    //Note: Init globals before anything else
    let init = Global::init(vec![], vec![]);
    
    let cam = Camera::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), (800, 600), 60, 10);
    cam.render();
}

