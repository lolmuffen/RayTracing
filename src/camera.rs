use std::time::{Duration, Instant};

use crate::gpu_structs::{CameraUniform, GpuMaterial, GpuSphere, GpuTriangle};
use crate::shape::{self, Shape};
use crate::{utils::sample_unit_square, vector::Vec3};
use crate::ray::Ray;
use bytemuck::Pod;
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

    pub async fn gpu_render(&self) {

        // ── 1. WGPU INIT ─────────────────────────────────────────────────────
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("No DX12 adapter found");

        println!("GPU: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("rt_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    ..Default::default()
                },
            )
            .await
            .expect("Failed to create device");

        // ── 2. SCENE DATA ────────────────────────────────────────────────────
        let global = get_GLOBAL();
        let objects = global.get_objects().unwrap();

        // Pack geometry + materials exactly as pack_scene() does in camera.rs
        let (gpu_spheres, gpu_triangles, gpu_materials) =
            Camera::pack_scene(objects);

        // Flatten BVH tree into the two arrays the shader reads
        let bvh = global.get_scene();
        let (bvh_shape_ids, bvh_nodes) = bvh.flatten();

        // ── 3. BUFFERS ───────────────────────────────────────────────────────

        // Helper: upload a &[T: Pod] as a read-only storage buffer
        fn upload_storage<T: Pod>(
            device: &wgpu::Device,
            data: &[T],
            label: &str,
        ) -> wgpu::Buffer {
            device.create_buffer(&wgpu::util::BufferDescriptor {}

            )
        }

        let material_buf   = upload_storage(&device, &gpu_materials,  "materials");
        let sphere_buf     = upload_storage(&device, &gpu_spheres,    "spheres");
        let triangle_buf   = upload_storage(&device, &gpu_triangles,  "triangles");
        let bvh_node_buf   = upload_storage(&device, &bvh_nodes,      "bvh_nodes");
        let bvh_ids_buf    = upload_storage(&device, &bvh_shape_ids,  "bvh_shape_ids");

        let (w, h) = (self.resolution.0 as usize, self.resolution.1 as usize);
        let pixel_count = w * h;

        // Accumulation buffer: vec4<f32> per pixel, persists across dispatches
        let accum_buf_size = (pixel_count * 4 * std::mem::size_of::<f32>()) as u64;
        let accum_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("accumulation_buf"),
            size:               accum_buf_size,
            usage:              wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Readback buffer: mappable, CPU reads accumulated pixels here
        let readback_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("readback_buf"),
            size:               accum_buf_size,
            usage:              wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Camera uniform buffer (updated every frame with new sample counts)
        let camera_uniform_size = std::mem::size_of::<CameraUniform>() as u64;
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("camera_uniform"),
            size:               camera_uniform_size,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── 4. SHADER ────────────────────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("path_tracer"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../path_tracer.wgsl").into(),
            ),
        });

        // ── 5. BIND GROUP LAYOUTS ────────────────────────────────────────────

        // group(0): all scene data + camera uniform (read-only)
        let bgl0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("bgl_scene"),
            entries: &[
                // binding 0: CameraUniform
                wgpu::BindGroupLayoutEntry {
                    binding:    0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                // bindings 1-5: storage read-only (materials, spheres, triangles, bvh_nodes, bvh_ids)
                wgpu::BindGroupLayoutEntry {
                    binding:    1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:    5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    count: None,
                },
            ],
        });

        // group(1): accumulation buffer (read-write)
        let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("bgl_accum"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            }],
        });

        // ── 6. BIND GROUPS ───────────────────────────────────────────────────

        let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("bg_scene"),
            layout:  &bgl0,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: material_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: sphere_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: triangle_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: bvh_node_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: bvh_ids_buf.as_entire_binding() },
            ],
        });

        let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("bg_accum"),
            layout:  &bgl1,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: accum_buf.as_entire_binding() },
            ],
        });

        // ── 7. PIPELINE ──────────────────────────────────────────────────────

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("rt_pipeline_layout"),
            bind_group_layouts:   &[&bgl0, &bgl1],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label:               Some("rt_pipeline"),
            layout:              Some(&pipeline_layout),
            module:              &shader,
            entry_point:         Some("main"),
            compilation_options: Default::default(),
            cache:               None,
        });

        // ── 8. WINDOW + RENDER LOOP ──────────────────────────────────────────
        let mut window = Window::new(
            "RayTracing GPU - Progressive",
            w,
            h,
            WindowOptions::default(),
        ).expect("Unable to open window");

        let samples_this_frame = self.samples_per_pixel;
        let mut total_samples:  u32 = 0;
        let mut frame_count:    u32 = 0;
        let mut pixel_buffer:   Vec<u32> = vec![0u32; pixel_count];
        let mut frame_time_avg  = std::time::Duration::ZERO;
        let print_every = 10;

        // Workgroup size matches @workgroup_size(8, 8, 1) in shader
        let wg_x = (self.resolution.0 + 7) / 8;
        let wg_y = (self.resolution.1 + 7) / 8;

        while window.is_open() && !window.is_key_down(Key::Escape) {
            frame_count += 1;
            let t0 = Instant::now();

            // Upload updated camera uniform with new sample counts
            let uniform = self.to_gpu_uniform(samples_this_frame, total_samples);
            queue.write_buffer(&camera_buf, 0, bytemuck::bytes_of(&uniform));

            // Dispatch compute shader
            let mut encoder = device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("frame_encoder") }
            );
            {
                let mut cpass = encoder.begin_compute_pass(
                    &wgpu::ComputePassDescriptor { label: Some("rt_pass"), timestamp_writes: None }
                );
                cpass.set_pipeline(&pipeline);
                cpass.set_bind_group(0, &bg0, &[]);
                cpass.set_bind_group(1, &bg1, &[]);
                cpass.dispatch_workgroups(wg_x, wg_y, 1);
            }
            // Copy accumulation buffer → readback buffer
            encoder.copy_buffer_to_buffer(&accum_buf, 0, &readback_buf, 0, accum_buf_size);
            queue.submit(std::iter::once(encoder.finish()));

            total_samples += samples_this_frame;

            // Map readback buffer and convert to u32 pixels
            {
                let slice = readback_buf.slice(..);
                let (tx, rx) = std::sync::mpsc::channel();
                slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).unwrap(); });
                device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(Duration::new(1, 0)) });
                rx.recv().unwrap().expect("map_async failed");

                let raw: &[f32] = bytemuck::cast_slice(&slice.get_mapped_range());
                let inv = 1.0 / total_samples as f32;

                // Each pixel is vec4<f32> = 4 f32s; we only use .xyz
                pixel_buffer
                    .par_iter_mut()
                    .enumerate()
                    .for_each(|(i, pixel)| {
                        let base = i * 4;
                        let color = Vec3::new(raw[base] * inv, raw[base+1] * inv, raw[base+2] * inv);
                        *pixel = Color::color_to_hex(Color::gamma_correct_color(&color));
                    });

                // Unmap happens when MappedRange drops
            }

            window.update_with_buffer(&pixel_buffer, w, h).ok();

            let elapsed = t0.elapsed();
            frame_time_avg += elapsed;
            if frame_count % print_every == 0 {
                println!(
                    "Frame {frame_count} | Samples: {total_samples} | \
                     Avg frame: {}ms",
                    frame_time_avg.as_millis() / print_every as u128
                );
                frame_time_avg = std::time::Duration::ZERO;
            }
        }
    }



    pub fn path_pixel_color(&self, mut current_ray: Ray) -> Color {
        let global = get_GLOBAL();
        let scene = global.get_scene();

        for _bounce in 0..self.depth {
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

            } 
            else {

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

    fn pack_scene(objects: &[Shape]) -> (Vec<GpuSphere>, Vec<GpuTriangle>, Vec<GpuMaterial>) {
        let mut gpu_spheres:   Vec<GpuSphere>   = Vec::new();
        let mut gpu_triangles: Vec<GpuTriangle> = Vec::new();
        let mut gpu_materials: Vec<GpuMaterial> = Vec::new();

        for shape in objects {
            let mat_id = gpu_materials.len() as u32;
            gpu_materials.push(shape.get_material().to_gpu_material());

            match shape {
                Shape::Sphere { sphere } => {
                    gpu_spheres.push(GpuSphere {
                        center:      sphere.position.to_array(),
                        radius:      sphere.radius,
                        material_id: mat_id,
                        _pad:        [0; 3],
                    });
                }
                Shape::Triangle { tri } => {
                    gpu_triangles.push(GpuTriangle {
                        p1: tri.p1.to_array(), _p1: 0.0,
                        p2: tri.p2.to_array(), _p2: 0.0,
                        p3: tri.p3.to_array(), _p3: 0.0,
                        normal:      tri.normal.to_array(),
                        material_id: mat_id,
                    });
                }
            }
        }

        (gpu_spheres, gpu_triangles, gpu_materials)
    }

    pub fn to_gpu_uniform(&self, samples_per_dispatch: u32, total_samples: u32) -> CameraUniform {
        CameraUniform {
            origin:               self.position.to_array(),
            focal_length:         self.focal_length,
            pixel_upper_left:     self.origin_pixel_upper_left.to_array(),
            focus_distance:       self.focus_distance,
            delta_u:              self.delta_u.to_array(),
            aperture_radius:      self.aperture_radius,
            delta_v:              self.delta_v.to_array(),
            _pad0:                0.0,
            right:                self.right.to_array(),
            _pad1:                0.0,
            up:                   self.up.to_array(),
            _pad2:                0.0,
            sun_direction:        self.sun_direction.to_array(),
            _pad3:                0.0,
            width:                self.resolution.0,
            height:               self.resolution.1,
            samples_per_dispatch,
            total_samples,
            max_depth:            self.depth,
            _pad4:                0,
            _pad5:                0,
            _pad6:                0,
        }
    }
}
