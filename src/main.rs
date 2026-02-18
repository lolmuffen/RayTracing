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


use rayon::vec;

use crate::light::SphereLight;
use crate::sphere::Sphere;
use crate::vector::Vec3;
use crate::camera::{Camera, Color};
use crate::utils::{GLOBAL, Global, get_GLOBAL};
use crate::material::{Lambertian, Metal};

fn main() {
    //Init global state for unique ID generation
    let init = Global::init();


    //Note: Init globals before anything else
    let material_ground = Box::new(Lambertian::new(1.0,Color::new(0.8, 0.8, 0.0)));

    /// Center sphere: blue diffuse material for classic matte appearance
    let material_center = Box::new(Lambertian::new(1.0,Color::new(0.1, 0.2, 0.5)));

    /// Left sphere: metallic with low roughness for mirror-like reflections
    let material_left = Box::new(Metal::new(1.0, Color::new(0.8, 0.8, 0.8), 0.0));

    /// Right sphere: metallic with high roughness for brushed metal appearance
    let material_right = Box::new(Metal::new(1.0, Color::new(0.8, 0.6, 0.0), 0.5));

    // =============================================================================
    // Scene Geometry Setup
    // =============================================================================
    // Create the geometric objects that make up our test scene.
    // Spheres are positioned to create an interesting composition.

    /// Ground plane: Large sphere positioned below the scene
    /// Acts as an infinite-looking ground due to its size relative to other objects
    let sphere1 = Sphere::new(Vec3 { x: 0.0, y: -101.0, z: -1.0 }, 100.5, Color::new(0.8, 0.8, 0.0), material_ground);

    /// Center sphere: Main subject of the scene
    let sphere2 = Sphere::new(Vec3 { x: 0.0, y: 0.0, z: -1.0 }, 0.5, Color::new(0.1, 0.2, 0.5), material_center);

    /// Left sphere: Demonstrates metallic reflection
    let sphere3 = Sphere::new(Vec3 { x: -1.0, y: 0.0, z: -1.0 }, 0.5, Color::new(0.8, 0.8, 0.8), material_left);

    /// Right sphere: Shows rougher metallic surface
    let sphere4 = Sphere::new(Vec3 { x: 1.0, y: 0.0, z: -1.0 }, 0.5, Color::new(0.8, 0.6, 0.0), material_right);

    // =============================================================================
    // Scene Assembly
    // =============================================================================
    // Add all objects to the scene container for intersection testing
    let objects: Vec<Box<dyn crate::shape::Shape + Send + Sync>> = vec![Box::new(sphere1), Box::new(sphere2), Box::new(sphere3), Box::new(sphere4)];

    let mut lights: Vec<Box<dyn crate::light::Light + Send + Sync>> = Vec::new();
    lights.push(Box::new(SphereLight::new(Vec3::new(1.0, 4.0, -1.0), 1.0, Color::new(1.0, 1.0, 1.0), 1.0)));
    

    // Set global state with our scene objects and lights for access during rendering
    get_GLOBAL().set_objects(objects);
    get_GLOBAL().set_lights(lights);
    get_GLOBAL().scene.set(crate::BVH::sceneBVH::new());

    /// Image dimensions and quality settings
    let width: i32 = 1080;                    // Full HD width
    let aspect_ratio: f32 = 16.0 / 9.0;       // Widescreen cinema aspect ratio
    let samples_per_pixel: u32 = 5;          // Anti-aliasing quality (higher = smoother)
    let max_depth: i32 = 10;                  // Maximum light bounces (higher = more accurate GI)
    let light_samples: u32 = 5;              // Number of samples for direct illumination (higher = softer shadows)

    // Create the virtual camera with specified parameters
    let cam = Camera::new(Vec3::zero(), Vec3::new(0.0, 0.0, -1.0), (width as u32, (width as f32 / aspect_ratio) as u32), 80, samples_per_pixel, max_depth as u32, light_samples);

    // =============================================================================
    // Rendering Execution
    // =============================================================================
    // Render the scene and output the result as a PPM image to stdout
    
    cam.render();
}

