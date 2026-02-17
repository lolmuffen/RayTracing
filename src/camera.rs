use crate::vector::Vec3;
use minifb::{Key, Window, WindowOptions};

pub type Color = Vec3;

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub resolution: (u32, u32),
    pub fov: u32,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, resolution: (u32, u32), fov: u32) -> Self {
        Camera {
            position,
            direction,
            resolution,
            fov,
        }
    }

    /// Very basic render: opens a window and exits on Escape or close.
    pub fn render(&self) {
        let (width, height) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let mut buffer: Vec<u32> = vec![0; width * height];

        let mut window = Window::new(
            "RayTracing - Basic Window",
            width,
            height,
            WindowOptions::default(),
        ).expect("Unable to open window");

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // Fill buffer with a simple color pattern (black)
            for pixel in buffer.iter_mut() {
                *pixel = 0xFF0F00; // black
            }

            // Update the window with the buffer
            window.update_with_buffer(&buffer, width, height).ok();
        }
    }
}