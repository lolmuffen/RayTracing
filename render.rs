use rayon::prelude::*;
use crate::hit::{HitRecord, Hittable, HittableList};
use crate::ray::Ray;
use crate::vector::Vec3;
use crate::utils::{sample_unit_square, Interval};
use crate::light::{Light};

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

    pub fn write_color(color: &Color) {
        let mut r = color.x;
        let mut g = color.y;
        let mut b = color.z;

        r = Color::linear_to_gamma(r);
        g = Color::linear_to_gamma(g);
        b = Color::linear_to_gamma(b);

        let rbyte = (INTENSITY.clamp(r) * 256.0) as u8;
        let gbyte = (INTENSITY.clamp(g) * 256.0) as u8;
        let bbyte = (INTENSITY.clamp(b) * 256.0) as u8;

       println!("{} {} {}", rbyte, gbyte, bbyte);
    }

    pub fn zero() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

pub struct Camera {
    pub aspect_ratio: f32,
    pub samples_per_pixel: u32,
    pub image_width: i32,
    pub depth: Interval,
    image_height: i32,
    center: Vec3,
    origin_pixel_upper_left: Vec3,
    delta_u: Vec3,
    delta_v: Vec3,
    pixel_sample_scale: f32,
    light_samples: u32,
}

impl Camera {
    pub fn render(&self, world: &HittableList, lights: &Vec<Light>) {
        let width = self.image_width;
        let height = self.image_height;
        let total_pixels = (width * height) as usize;

        println!("P3\n{} {}\n255\n", width, height);

        let pixels: Vec<(i32, i32)> = (0..height).flat_map(|j| (0..width).map(move |i| (i, j))).collect();

        eprintln!("Rendering {} pixels...", total_pixels);

        let pixel_colors: Vec<Color> = pixels
            .par_iter()
            .map(|&(i, j)| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);

                for _sample in 0..self.samples_per_pixel {
                    let mut ray = self.get_sample_ray(i, j);
                    pixel_color += self.ray_color_from_path(
                        &mut ray,
                        self.depth.max as i32,
                        world,
                        lights
                    );
                }

                pixel_color * self.pixel_sample_scale
            })
            .collect();

        eprintln!("Rendering complete! Writing output...");

        for color in pixel_colors {
            Color::write_color(&color);
        }
    }

    fn ray_color_from_path(&self, initial_ray: &mut Ray, depth: i32, world: &HittableList, lights: &Vec<Light>) -> Color {
        let mut current_ray = initial_ray;
        let mut ray_color = Color::new(1.0, 1.0, 1.0);

        for _bounce in 0..depth {
            if let Some(hit_record) = world.hit(&current_ray, &Interval::new(0.001, f32::INFINITY)) {
                let light_intensity = self.direct_illumination(&hit_record, world, lights);

                if let Some(scatter_result) = hit_record.mat.scatter(&mut current_ray, &hit_record) {
                    *current_ray = scatter_result.scattered;
                    ray_color = ray_color.component_mul(scatter_result.attenuation) + light_intensity;
                } else {
                    return Color::new(0.0, 0.0, 0.0);
                }
            } else {
                return ray_color.component_mul(self.background_color(&current_ray));
            }
        }

        Color::new(0.0, 0.0, 0.0)
    }

    fn direct_illumination(&self, hit_record: &HitRecord, world: &HittableList, lights: &Vec<Light>) -> Color {
        let pos = hit_record.p;
        let mut light_intensity = Color::new(0.0, 0.0, 0.0);

        for light in lights {
            for _sample in 0..self.light_samples {
                let sample_ray = light.sample_sphere_light(pos);
                if let Some(_) = world.hit(&sample_ray.normalized(), &Interval::new(0.001, f32::INFINITY)) {
                    light_intensity -= light.sphere.mat.emitted() / (self.light_samples as f32 * sample_ray.length() * sample_ray.length());
                }
                else {
                    light_intensity += light.sphere.mat.emitted() / (self.light_samples as f32 * sample_ray.length() * sample_ray.length());
                }
            }
        }

        light_intensity

    }

    fn background_color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction.normalized();
        let t = 0.5 * (unit_direction.y + 1.0);

        Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    }

    pub fn new(width: i32, aspect_ratio: f32, samples: u32, max_depth: i32) -> Self {
        let height = (width as f32 / aspect_ratio) as i32;

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (width as f32 / height as f32);

        let camera_center = Vec3::new(0.0, 0.0, 0.0);

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        let delta_u = viewport_u / width as f32;
        let delta_v = viewport_v / height as f32;

        let viewport_upper_left = camera_center
            - Vec3::new(0.0, 0.0, focal_length)
            - viewport_u / 2.0
            - viewport_v / 2.0;

        let origin_pixel_upper_left = viewport_upper_left + (delta_u * 0.5) + (delta_v * 0.5);

        Camera {
            aspect_ratio,
            samples_per_pixel: samples,
            image_width: width,
            depth: Interval::new(0.0, max_depth as f32),
            image_height: height,
            center: Vec3::new(0.0, 0.0, 0.0),
            origin_pixel_upper_left,
            delta_u,
            delta_v,
            pixel_sample_scale: 1.0 / samples as f32,
            light_samples: 5,
        }
    }

    pub fn get_sample_ray(&self, i: i32, j: i32) -> Ray {
        let offset = sample_unit_square();

        let pixel_sample = self.origin_pixel_upper_left
            + (self.delta_u * (i as f32 + offset.x))
            + (self.delta_v * (j as f32 + offset.y));

        let ray_direction = pixel_sample - self.center;

        Ray::new(self.center, ray_direction)
    }
}