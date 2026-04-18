mod vector;
mod utils;
mod ray;
mod sphere;
mod material;
mod intersection;
mod shape;
mod BVH;
mod camera;
mod triangle;
mod triangle_bvh;
mod objectloader;

use crate::vector::Vec3;
use crate::camera::{Camera, Color};
use crate::utils::{Global, get_GLOBAL};
use crate::material::Material;
use crate::shape::Shape;


fn main() {
    //Init global state for unique ID generation
    Global::init();


    // =============================================================================
    // Scene Geometry Setup
    // =============================================================================
    let material_ground = Material::lambertian(1.0,Color::new(0.8, 0.8, 0.0));
    let material_center = Material::specular(Color::new(0.1, 0.2, 0.5), 1.0, 0.0, 0.8, );
    let material_left = Material::metal(1.0, Color::new(0.8, 0.8, 0.8), 0.0);
    let material_right = Material::glass(1.5, 2.0, Color::new(0.8, 0.6, 0.4));

    let sphere1 = Shape::sphere(Vec3 { x: 0.0, y: -101.0, z: -1.0 }, 100.5, material_ground);
    let sphere2 = Shape::sphere(Vec3 { x: 0.0, y: 0.0, z: -1.0 }, 0.6, material_center);
    let sphere3 = Shape::sphere(Vec3 { x: -1.0, y: 0.0, z: -1.0 }, 0.5, material_left);
    let sphere4 = Shape::sphere(Vec3 { x: 1.0, y: 0.0, z: -1.0 }, 0.5, material_right);

    let tris = objectloader::load_obj("knife chess piece.obj", material_left)
        .expect("Failed to load OBJ");


    let mesh_bvh = Shape::triangle_mesh(&tris, Vec3::new(0.0, 0.0, -0.0),  0.3);
    
    let light = Shape::sphere(Vec3 { x: 3.0, y: 5.0, z: -0.0 }, 1.0, Material::emissive(Color::new(4.0, 3.0, 2.0), 1.0));

    // =============================================================================
    // Scene Assembly
    // =============================================================================
    let objects: Vec<Shape> = vec![sphere1, sphere2, sphere3, sphere4, mesh_bvh, light];


    // Set global state with our scene objects and lights for access during rendering
    let _ = get_GLOBAL().set_objects(objects);
    let _ = get_GLOBAL().scene.set(crate::BVH::sceneBVH::new());


    let width: i32 = 1080;
    let aspect_ratio: f32 = 16.0 / 9.0;
    let fov = 100;
    let samples_per_pixel: u32 = 3;
    let max_depth: i32 = 8;
    // Depth of field (distance focusing) parameters
    let focus_distance: f32 = 2.0;
    let aperture: f32 = 0.0001;          // Camera aperture diameter (0.0 = pinhole, larger = more blur)

    // Create the virtual camera with specified parameters and sun direction
    let sun_direction = Vec3::new(1.0, 2.0, -1.0);

    let mut cam = Camera::new(Vec3::new(0.5, 0.0, 0.5), Vec3::new(0.0, 0.0, -1.0), (width as u32, (width as f32 / aspect_ratio) as u32), fov, samples_per_pixel, max_depth as u32, focus_distance, aperture, sun_direction);
    
    cam.render();
}
