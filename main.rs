//! main.rs - Complete documentation
//! A Monte Carlo path tracing ray tracer implementation in Rust.
//!
//! This ray tracer implements physically-based rendering using Monte Carlo
//! integration to solve the rendering equation. It supports:
//!
//! ## Features
//! - **Multiple Material Types**: Lambertian diffuse, metal with roughness, emissive lights
//! - **Global Illumination**: Recursive ray bouncing simulates light transport
//! - **Anti-aliasing**: Multiple samples per pixel reduce jagged edges
//! - **Soft Shadows**: Area lights create realistic shadow penumbras
//! - **HDR Rendering**: High dynamic range with gamma correction
//!
//! ## Architecture
//! The ray tracer follows a modular design:
//! - `ray`: Ray representation and operations
//! - `vector`: 3D vector math and utilities
//! - `hit`: Intersection testing framework
//! - `material`: Surface material definitions and light interaction
//! - `sphere`: Sphere geometry primitive
//! - `light`: Emissive light sources
//! - `render`: Camera and rendering pipeline
//! - `utils`: Mathematical utilities and random sampling
//!
//! ## Usage
//! ```bash
//! rustc main.rs
//! ./main > output.ppm
//! ```
//!
//! The program outputs a PPM image file to stdout. Redirect to a file
//! and open with any image viewer that supports PPM format.
//!
//! ## Rendering Algorithm
//! 1. For each pixel, generate multiple anti-aliasing samples
//! 2. Cast a ray from camera through each sample point
//! 3. Recursively trace ray bounces through the scene:
//!    - Test intersection with all scene objects
//!    - If hit: accumulate material color and scatter new ray
//!    - If light hit: return accumulated color × light emission
//!    - If background hit: return accumulated color × sky color
//! 4. Average all samples for final pixel color
//! 5. Apply gamma correction and output as 8-bit RGB

mod ray;
mod hit;
mod sphere;
mod render;
mod utils;
mod material;
mod vector;
mod light;

use hit::HittableList;
use sphere::Sphere;
use crate::light::Light;
use crate::render::{Camera, Color};
use crate::material::{Dielectric, Lambertian, Metal};
use crate::vector::Vec3;

/// Entry point for the ray tracer application.
///
/// Sets up a test scene with spheres of different materials and renders
/// it using a virtual camera. The scene demonstrates various ray tracing
/// effects including diffuse and specular reflection, lighting, and shadows.
fn main() {
    // =============================================================================
    // Material Creation
    // =============================================================================
    // Create materials that define how surfaces interact with light.
    // Each material implements different physical properties.

    /// Large diffuse sphere acts as ground plane with yellow-green color
    let material_ground = Box::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    /// Center sphere: blue diffuse material for classic matte appearance
    let material_center = Box::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));

    /// Left sphere: metallic with low roughness for mirror-like reflections
    let material_left = Box::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.0));

    /// Right sphere: metallic with high roughness for brushed metal appearance
    let material_right = Box::new(Metal::new(Color::new(0.8, 0.6, 0.0), 1.0));

    // =============================================================================
    // Scene Geometry Setup
    // =============================================================================
    // Create the geometric objects that make up our test scene.
    // Spheres are positioned to create an interesting composition.

    /// Ground plane: Large sphere positioned below the scene
    /// Acts as an infinite-looking ground due to its size relative to other objects
    let sphere1 = Sphere::new(0.0, -100.5, -1.0, 100.0, material_ground);

    /// Center sphere: Main subject of the scene
    let sphere2 = Sphere::new(0.0, 0.0, -1.0, 0.5, material_center);

    /// Left sphere: Demonstrates metallic reflection
    let sphere3 = Sphere::new(-1.0, 0.0, -1.0, 0.5, material_left);

    /// Right sphere: Shows rougher metallic surface
    let sphere4 = Sphere::new(1.0, 0.0, -1.0, 0.5, material_right);

    // =============================================================================
    // Scene Assembly
    // =============================================================================
    // Add all objects to the scene container for intersection testing

    let mut world = HittableList::new();
    world.add(Box::new(sphere1));   // Ground
    world.add(Box::new(sphere2));   // Center sphere
    world.add(Box::new(sphere3));   // Left sphere
    world.add(Box::new(sphere4));   // Right sphere

    let mut lights = Vec::<Light>::new();
    lights.push(Light::new(Vec3::new(1.0, 4.0, -1.0), 1.0, Color::new(1.0, 1.0, 1.0), 1.0));

    // =============================================================================
    // Rendering Configuration
    // =============================================================================
    // Set up camera and rendering parameters for the final image

    /// Image dimensions and quality settings
    let width: i32 = 1080;                    // Full HD width
    let aspect_ratio: f32 = 16.0 / 9.0;       // Widescreen cinema aspect ratio
    let samples_per_pixel: u32 = 10;          // Anti-aliasing quality (higher = smoother)
    let max_depth: i32 = 50;                  // Maximum light bounces (higher = more accurate GI)

    // Create the virtual camera with specified parameters
    let cam = Camera::new(width, aspect_ratio, samples_per_pixel, max_depth);

    // =============================================================================
    // Rendering Execution
    // =============================================================================
    // Render the scene and output the result as a PPM image to stdout

    cam.render(&world, &lights);
}