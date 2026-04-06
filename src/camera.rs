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
        
        (rbyte, gbyte, bbyte)
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
        
        let fov_rad = (fov as f32).to_radians();
        let h = (fov_rad / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * (width as f32 / height as f32);
        
        let viewport_u = right * viewport_width;
        let viewport_v = -up * viewport_height;
        
        let delta_u = viewport_u / width as f32;
        let delta_v = viewport_v / height as f32;
        
        let viewport_upper_left = center - (forward * focal_length) - (viewport_u / 2.0) - (viewport_v / 2.0);
        let origin_pixel_upper_left = viewport_upper_left + (delta_u * 0.5) + (delta_v * 0.5);

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

    pub fn update_position(&mut self, position: Vec3, direction: Vec3) {
        self.position = position;
        self.direction = direction;
        self.center = position;

        let (width, height) = self.resolution;

        self.focal_length = (position - direction).length();

        let world_up = Vec3::new(0.0, 1.0, 0.0);

        self.forward = (position - direction).normalize();
        self.right = world_up.cross(self.forward).normalize();
        self.up = self.forward.cross(self.right).normalize();

        let fov_rad = (self.fov as f32).to_radians();
        let h = (fov_rad / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * (width as f32 / height as f32);

        let viewport_u = self.right * viewport_width;
        let viewport_v = -self.up * viewport_height;

        self.delta_u = viewport_u / width as f32;
        self.delta_v = viewport_v / height as f32;

        let viewport_upper_left = self.center
            - (self.forward * self.focal_length)
            - (viewport_u / 2.0)
            - (viewport_v / 2.0);

        self.origin_pixel_upper_left =
            viewport_upper_left + (self.delta_u * 0.5) + (self.delta_v * 0.5);
    }

    pub fn render(&mut self) {
        let (width, height) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let pixel_count = width * height;

        let mut window = Window::new(
            "RayTracing - Progressive",
            width,
            height,
            WindowOptions::default(),
        ).expect("Unable to open window");

        let mut accumulation_buffer: Vec<Vec3> = vec![Vec3::new(0.0, 0.0, 0.0); pixel_count];
        let mut pixel_buffer: Vec<u32> = vec![0u32; pixel_count];
        let mut new_samples: Vec<Vec3> = vec![Vec3::new(0.0, 0.0, 0.0); pixel_count];

        let mut total_samples: u32 = 0;
        let samples_this_frame = self.samples_per_pixel;
        let mut frame_count: u32 = 0;
        let frame_count_before_print = 10;
        let mut frame_time_average = Duration::new(0, 0);

        let move_speed: f32 = 0.1;
        let rotate_angle: f32 = 0.1;

        while window.is_open() && !window.is_key_down(Key::Escape) {

            // --- Input handling ---
            let mut frame_dirty = false;

            let keys = window.get_keys();

            for key in keys {
                let m: Option<()> = match key {
                    Key::W => {
                        self.position -= self.forward * move_speed;
                        self.direction -= self.forward * move_speed;
                        Some(())
                    },
                    Key::A => {
                        self.position -= self.right * move_speed;
                        self.direction -= self.right * move_speed;
                        Some(())
                    },
                    Key::S => {
                        self.position += self.forward * move_speed;
                        self.direction += self.forward * move_speed;
                        Some(())
                    },
                    Key::D => {
                        self.position += self.right * move_speed;
                        self.direction += self.right * move_speed;
                        Some(())
                    },
                    Key::Space => {
                        self.position += self.up * move_speed;
                        self.direction += self.up * move_speed;
                        Some(())
                    },
                    Key::LeftShift => {
                        self.position -= self.up * move_speed;
                        self.direction -= self.up * move_speed;
                        Some(())
                    }, 
                    Key::Up => {
                        if self.direction.dot(self.up) < 0.99 {
                            self.direction = self.direction.rotate_around(self.right, rotate_angle);
                            Some(())
                        }
                        else {
                            println!("No UP");
                            None
                        }
                        
                    }
                    Key::Down => {
                        if self.direction.dot(-self.up) < 0.99 {
                            self.direction = self.direction.rotate_around(self.right, -rotate_angle);
                            Some(())
                        }
                        else {
                            println!("No Down");
                            None
                        }
                    }
                    
                    
                    _ => {None}
                };

                if m.is_some() {
                    frame_dirty = true;
                } 
            }


            if frame_dirty {

                self.update_position(self.position, self.direction);

                accumulation_buffer.fill(Vec3::new(0.0, 0.0, 0.0));
                pixel_buffer.fill(0);
                total_samples = 0;
                frame_count = 0;
                frame_time_average = Duration::new(0, 0);
            }
           
            frame_count += 1;
            let frame_start = Instant::now();

            new_samples
                .par_chunks_mut(width)
                .enumerate()
                .for_each(|(y, row)| {
                    for x in 0..width {
                        let mut accumulated = Color::new(0.0, 0.0, 0.0);
                        for _ in 0..samples_this_frame {
                            let ray = Camera::get_sample_ray_raw(
                                x as u32, y as u32,
                                self.origin_pixel_upper_left, self.delta_u, self.delta_v,
                                self.center, self.right, self.up, self.aperture_radius, self.focus_distance,
                            );
                            accumulated += Camera::path_pixel_color_raw(ray, self.depth, self.sun_direction);
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

            if frame_count.is_multiple_of(frame_count_before_print) {
                println!(
                    "Frame {frame_count} | Total samples: {total_samples} | \
                    Avg frame time: {}ms",
                    frame_time_average.as_millis() / frame_count_before_print as u128
                );
                frame_time_average = Duration::new(0, 0);
            }
        }
    }

    // Free function version used inside rayon closure (no &self borrow needed).
    fn get_sample_ray_raw(
        i: u32, j: u32,
        origin_pixel_upper_left: Vec3,
        delta_u: Vec3,
        delta_v: Vec3,
        center: Vec3,
        right: Vec3,
        up: Vec3,
        aperture_radius: f32,
        focus_distance: f32,
    ) -> Ray {
        let offset = sample_unit_square();

        let pixel_sample = origin_pixel_upper_left
            + (delta_u * (i as f32 + offset.x))
            + (delta_v * (j as f32 + offset.y));

        let aperture_sample = Vec3::random_vec_on_circle();
        let aperture_offset = (right * aperture_sample.x + up * aperture_sample.y) * aperture_radius;
        let ray_origin = center + aperture_offset;

        let ray_direction_to_focal = (pixel_sample - center).normalize();
        let focal_plane_point = center + ray_direction_to_focal * focus_distance;
        let final_direction = focal_plane_point - ray_origin;

        Ray::new(ray_origin, final_direction, Color::new(1.0, 1.0, 1.0))
    }

    // Free function version used inside rayon closure (no &self borrow needed).
    fn path_pixel_color_raw(mut current_ray: Ray, depth: u32, sun_direction: Vec3) -> Color {
        let global = get_GLOBAL();
        let scene = global.get_scene();

        for _bounce in 0..depth {
            if let Some(hit_record) = scene.traverse(&current_ray, global) {
                let material = hit_record.hitdata.as_ref()
                    .and_then(|h| h.material)
                    .unwrap_or_else(|| {
                        global
                            .get_object_by_id(hit_record.object_id.unwrap())
                            .unwrap()
                            .get_material()
                    });

                if material.is_emissive() {
                    let scattered = material.scatter(&current_ray, &hit_record);
                    return scattered.color;
                }

                current_ray = material.scatter(&current_ray, &hit_record);
            } else {
                return current_ray.color * Camera::background_color_raw(&current_ray, sun_direction);
            }
        }

        Color::zero()
    }

    fn background_color_raw(ray: &Ray, sun_direction: Vec3) -> Color {
        let unit_direction = ray.direction.normalize();
        let t = 0.5 * (unit_direction.y + 1.0);
        let sky_color = Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t;
        let sun_dot = unit_direction.dot(sun_direction).powf(10.0).max(0.0);
        let sun_disk = 1.0 / (1.0 / (Color::new(1.0, 0.95, 0.7) * sun_dot.powf(20.0))) * 2.0;
        (sky_color + sun_disk) / 2.0
    }
}