use std::time::{Duration, Instant};

use crate::{utils::sample_unit_square, vector::Vec3};
use crate::ray::Ray;
use minifb::{Key, Window, WindowOptions};
use crate::utils::{Interval, get_GLOBAL};
use rayon::prelude::*;

pub type Color = Vec3;

const INTENSITY: Interval = Interval::new(0.0, 0.999);

impl Color {
    pub fn linear_to_gamma(linear_component: f32) -> f32 {
        if linear_component > 0.0 {
            linear_component.sqrt()
        } else {
            0.0
        }
    }

    pub fn gamma_correct_color(color: &Color) -> (u8, u8, u8) {
        let mut r = color.x;
        let mut g = color.y;
        let mut b = color.z;

        r = Color::linear_to_gamma(r);
        g = Color::linear_to_gamma(g);
        b = Color::linear_to_gamma(b);

        let rbyte = (INTENSITY.clamp(r) * 256.0) as u8;
        let gbyte = (INTENSITY.clamp(g) * 256.0) as u8;
        let bbyte = (INTENSITY.clamp(b) * 256.0) as u8;
        
        return (rbyte, gbyte, bbyte);
    }

    pub fn color_to_hex(color: (u8, u8, u8)) -> u32 {
        let (r, g, b) = color;
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    pub fn zero() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub resolution: (u32, u32),
    pub fov: u32,
    pub samples_per_pixel: u32,
    pub depth: u32,
    pub center: Vec3,
    pub origin_pixel_upper_left: Vec3,
    pub delta_u: Vec3,
    pub delta_v: Vec3,
    pub focal_length: f32,
    pub focus_distance: f32,
    pub aperture_radius: f32,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub sun_direction: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, resolution: (u32, u32), fov: u32, samples: u32, num_bounces: u32, focus_distance: f32, aperture: f32, sun_direction: Vec3) -> Self {
        let center = position;
        let (width, height) = resolution;

        let focal_length = (position - direction).length();
    
        let world_up = Vec3::new(0.0, 1.0, 0.0);

        let forward = (position - direction).normalize();
        let right = world_up.cross(forward).normalize();
        let up = forward.cross(right).normalize();
        
        // Calculate FOV in radians
        let fov_rad = (fov as f32).to_radians();
        let h = (fov_rad / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * (width as f32 / height as f32);
        
        // Calculate pixel delta vectors
        let viewport_u = right * viewport_width;
        let viewport_v = -up * viewport_height;
        
        let delta_u = viewport_u / width as f32;
        let delta_v = viewport_v / height as f32;
        
        // Calculate the upper left corner of the pixel grid
        let viewport_upper_left = center - (forward * focal_length) - (viewport_u / 2.0) - (viewport_v / 2.0);
        let origin_pixel_upper_left = viewport_upper_left + (delta_u * 0.5) + (delta_v * 0.5);

        // Aperture radius is half the aperture diameter
        let aperture_radius = aperture / 2.0;
        
        Camera {
            position,
            direction,
            resolution,
            fov,
            samples_per_pixel: samples,
            depth: num_bounces,
            center,
            origin_pixel_upper_left,
            delta_u,
            delta_v,
            focal_length,
            focus_distance,
            aperture_radius,
            forward,
            right,
            up,
            sun_direction: sun_direction.normalize(),
        }
    }

    /// Render: opens a window and generates rays for each pixel using multi-threaded 2D pixel processing.
    /// Each thread can safely write to its own pixel using 2D coordinates (x, y).
    pub fn render(&self) {
        let (width, height) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let pixel_count = width * height;

        let mut window = Window::new(
            "RayTracing - Progressive",
            width,
            height,
            WindowOptions::default(),
        ).expect("Unable to open window");

        // Each pixel accumulates raw (linear) color across all frames
        let mut accumulation_buffer: Vec<Vec3> = vec![Vec3::new(0.0, 0.0, 0.0); pixel_count];
        let mut pixel_buffer: Vec<u32> = vec![0u32; pixel_count];
        let mut new_samples: Vec<Vec3> = vec![Vec3::new(0.0, 0.0, 0.0); pixel_count];

        let mut total_samples: u32 = 0;        // total samples accumulated so far
        let samples_this_frame = self.samples_per_pixel; // how many new samples to add each frame
        let mut frame_count: u32 = 0;
        let frame_count_before_print = 10;
        let mut frame_time_average = Duration::new(0, 0);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            frame_count += 1;
            let frame_start = Instant::now();

            // --- Step 1: accumulate new samples into a per-row delta buffer ---
            // We collect new contributions in a separate vec so rayon can work
            // without touching accumulation_buffer (which isn't Send across chunks easily)

            new_samples
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(y, row)| {
                    for x in 0..width {
                        let mut accumulated = Color::new(0.0, 0.0, 0.0);
                        for _ in 0..samples_this_frame {
                            let ray = self.get_sample_ray(x as u32, y as u32);
                            accumulated += self.path_pixel_color(ray);
                        }
                        row[x] = accumulated;
                    }
                });

            total_samples += samples_this_frame;
            let inv_total = 1.0 / total_samples as f32;

            accumulation_buffer
                .par_iter_mut()
                .zip(new_samples.par_iter())
                .zip(pixel_buffer.par_iter_mut())
                .for_each(|((acc, new), pixel)| {
                    *acc += *new;
                    *pixel = Color::color_to_hex(Color::gamma_correct_color(&(*acc * inv_total)));
                });
            

            window.update_with_buffer(&pixel_buffer, width, height).ok();

            let frame_time = frame_start.elapsed();
            frame_time_average += frame_time;

            if frame_count % frame_count_before_print == 0 {
                println!(
                    "Frame {frame_count} | Total samples: {total_samples} | \
                    Avg frame time: {}ms",
                    frame_time_average.as_millis() / frame_count_before_print as u128
                );
                frame_time_average = Duration::new(0, 0);
            }
        }
    }

    pub fn path_pixel_color(&self, mut current_ray: Ray) -> Color {
        let global = get_GLOBAL();
        let scene = global.get_scene();
        let depth = global.get_depth_limit();

        for _bounce in 0..depth {
            if let Some(hit_record) = scene.traverse(&current_ray, global) {
                let material = global
                    .get_object_by_id(hit_record.object_id.unwrap())
                    .unwrap()
                    .get_material();

                if material.is_emissive() {
                    // Check emissive BEFORE scattering, return current throughput * emission
                    let scattered = material.scatter(&current_ray, &hit_record);
                    return scattered.color;
                }

                let scattered_ray = material.scatter(&current_ray, &hit_record);
                current_ray = scattered_ray;
            } else {
                return current_ray.color * self.background_color(&current_ray);
            }
        }

        Color::zero()  // Ray exceeded depth limit — return black, not accumulated color
    } 

    fn background_color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction.normalize();
        let t = 0.5 * (unit_direction.y + 1.0);
        let sky_color = Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t;
        
        // Add sun glow to background
        let sun_dot = unit_direction.dot(self.sun_direction).powf(10.0).max(0.0);
        let sun_disk = 1.0 / ( 1.0 / (Color::new(1.0, 0.95, 0.7) * sun_dot.powf(20.0))) * 2.0;
        
        (sky_color + sun_disk ) / 2.0
    }

    pub fn get_sample_ray(&self, i: u32, j: u32) -> Ray {
        let offset = sample_unit_square();

        let pixel_sample = self.origin_pixel_upper_left
            + (self.delta_u * (i as f32 + offset.x))
            + (self.delta_v * (j as f32 + offset.y));

        // Depth of field: generate a random point on the aperture disk
        let aperture_sample = Vec3::random_vec_on_circle();
        let aperture_offset = (self.right * aperture_sample.x + self.up * aperture_sample.y) * self.aperture_radius;

        // The ray origin is offset from the camera center based on the aperture sample
        let ray_origin = self.center + aperture_offset;

        // Calculate the focal plane point: Cast a ray from center through pixel_sample at focus_distance
        let ray_direction_to_focal = (pixel_sample - self.center).normalize();
        let focal_plane_point = self.center + ray_direction_to_focal * self.focus_distance;

        // The actual ray direction goes from the offset origin to the focal plane point
        let final_direction = focal_plane_point - ray_origin;
        let ray_color = Color::new(1.0, 1.0, 1.0); // Default white color for rays

        Ray::new(ray_origin, final_direction, ray_color)
    }

}