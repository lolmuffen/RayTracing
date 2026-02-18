use crate::vector::Vec3;
use minifb::{Key, Window, WindowOptions};
use crate::utils::Interval;
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
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, resolution: (u32, u32), fov: u32, samples: u32) -> Self {
        Camera {
            position,
            direction,
            resolution,
            fov,
            samples_per_pixel: samples,
        }
    }

    /// Very basic render: opens a window and exits on Escape or close.
    pub fn render(&self) {
        let (width, height) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let mut buffer: Vec<u32> = vec![0; width * height]; // Simple buffer for pixel data

        let mut window = Window::new(
            "RayTracing - Basic Window",
            width,
            height,
            WindowOptions::default(),
        ).expect("Unable to open window");

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // Fill buffer in parallel with a simple color gradient
            buffer.par_iter_mut().enumerate().for_each(|(i, pixel)| {
                let r = (i * 255 / (width * height)) as u32;
                let g = r;
                let b = r;

                *pixel = Color::color_to_hex(Color::gamma_correct_color(&Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)));
            });

            // Update the window with the buffer
            window.update_with_buffer(&buffer, width, height).ok();
        }
        




    }
}