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
    pub light_samples: u32,
    pub focal_length: f32,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, resolution: (u32, u32), fov: u32, samples: u32, num_bounces: u32, light_samples: u32) -> Self {
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
            light_samples,
            focal_length
        }
    }

    /// Render: opens a window and generates rays for each pixel using multi-threaded 2D pixel processing.
    /// Each thread can safely write to its own pixel using 2D coordinates (x, y).
    pub fn render(&self) {
        let (width, height) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let buffer: Vec<u32> = vec![0; width * height]; // Simple buffer for pixel data
        let buffer = std::sync::Arc::new(std::sync::Mutex::new(buffer));

        let mut window = Window::new(
            "RayTracing - Basic Window",
            width,
            height,
            WindowOptions::default(),
        ).expect("Unable to open window");

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // Process pixels in parallel using 2D coordinates
            let samples = self.samples_per_pixel;
            let pixel_data: Vec<(usize, usize, u32)> = (0..height)
                .into_par_iter()
                .flat_map(|y| {
                    (0..width)
                        .into_iter()
                        .map(|x| {
                            // Accumulate multiple jittered samples per pixel
                            let mut accumulated = Color::zero();
                            for _s in 0..samples {
                                let ray = self.get_sample_ray(x as u32, y as u32);
                                accumulated += self.path_pixel_color(ray);
                            }

                            // Average the samples
                            let avg_color = accumulated / samples as f32;

                            // Gamma-correct and convert to hex
                            let hex_color = Color::color_to_hex(Color::gamma_correct_color(&avg_color));
                            (x, y, hex_color)
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            // Write pixel data to buffer (thread-safe approach)
            if let Ok(mut buf) = buffer.lock() {
                for (x, y, color) in pixel_data {
                    let index = y * width + x;
                    buf[index] = color;
                }
            }

            // Update the window with the buffer
            if let Ok(buf) = buffer.lock() {
                window.update_with_buffer(&buf, width, height).ok();
            }
        }
    }

    pub fn path_pixel_color(&self, mut current_ray: Ray) -> Color {

        for _bounce in 0..get_GLOBAL().get_depth_limit() {
            if let Some(hit_record) = get_GLOBAL().get_scene().traverse(&current_ray) {
                
                let scattered_ray = get_GLOBAL().get_object_by_id(hit_record.object_id.unwrap()).unwrap().get_material().scatter(&current_ray, &hit_record);
                current_ray = scattered_ray;
                if get_GLOBAL().get_object_by_id(hit_record.object_id.unwrap()).unwrap().get_material().is_emissive() {

                    return current_ray.color * get_GLOBAL().get_object_by_id(hit_record.object_id.unwrap()).unwrap().get_material().emitted();

                }
            }
            else 
            {
                return current_ray.color * self.background_color(&current_ray);
            }
        }

        Color::new(0.0, 0.0, 0.0)
    } 

    fn background_color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction.normalize();
        let t = 0.5 * (unit_direction.y + 1.0);

        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }

    pub fn get_sample_ray(&self, i: u32, j: u32) -> Ray {
        let offset = sample_unit_square();

        let pixel_sample = self.origin_pixel_upper_left
            + (self.delta_u * (i as f32 + offset.x))
            + (self.delta_v * (j as f32 + offset.y));

        let ray_direction = pixel_sample - self.center ;
        let ray_color = Color::new(1.0, 1.0, 1.0); // Default white color for rays

        Ray::new(self.center, ray_direction, ray_color)
    }

}