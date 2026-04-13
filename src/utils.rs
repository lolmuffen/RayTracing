//! utils.rs
//! Mathematical utilities and helper functions for ray tracing operations.
//!
//! This module provides essential mathematical building blocks used throughout
//! the ray tracer including random number generation, coordinate transformations,
//! and interval arithmetic for robust intersection testing.

use rand::RngExt;
use crate::{shape::Shape, vector::Vec3};
use std::sync::{OnceLock};
use std::sync::atomic::{AtomicU32, Ordering};
use crate::BVH::sceneBVH;
use crate::triangle::Triangle;
use crate::ray::Ray;


// ---------------------------------------------------------------------------
// Möller–Trumbore ray/triangle intersection
// ---------------------------------------------------------------------------

pub fn moller_trumbore(ray: &Ray, tri: &Triangle) -> Option<(f32, Vec3)> {
    let e1 = tri.p2 - tri.p1;
    let e2 = tri.p3 - tri.p1;

    let h   = ray.direction.cross(e2);
    let det = e1.dot(h);

    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // Ray is parallel to triangle
    }

    let inv_det = 1.0 / det;
    let s = ray.origin - tri.p1;
    let u = inv_det * s.dot(h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(e1);
    let v = inv_det * ray.direction.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = inv_det * e2.dot(q);
    if t > f32::EPSILON {
        Some((t, ray.origin + ray.direction * t))
    } else {
        None
    }
}

// =============================================================================
// Random Number Generation
// =============================================================================

/// Generates a random floating-point number in the range [0, 1).
pub fn random_double() -> f32 {
    let mut rng = rand::rng();
    let x: f32 = rng.random();
    x
}

pub fn random_double_range(min: f32, max: f32) -> f32 {
    let mut rng = rand::rng();
    let y: f32 = rng.random_range(min..max);
    y
}

/// Generates a random 2D offset within a unit square for anti-aliasing.

pub fn sample_unit_square() -> Vec3 {
    Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
}

// =============================================================================
// Interval Arithmetic
// =============================================================================

/// A mathematically empty interval where max < min.
pub const EMPTY: Interval = Interval::new(f32::INFINITY, f32::NEG_INFINITY);

/// An interval containing all possible real numbers.
pub const ALL: Interval = Interval::new(f32::NEG_INFINITY, f32::INFINITY);

#[derive(Debug, Clone, Copy)]
pub struct Interval {

    pub min: f32,

    pub max: f32,
}

impl Interval {

    pub const fn new(min: f32, max: f32) -> Interval {
        Interval { min, max }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn contains(&self, x: f32) -> bool {
        self.min <= x && x <= self.max
    }


    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }


    pub fn clamp(&self, x: f32) -> f32 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }
}

// =============================================================================
// Global Variables and Constants
// =============================================================================


pub struct Global { 
    pub global_object_id: AtomicU32,
    pub global_object_list: OnceLock<Vec<Shape>>,
    pub BVH_DEPTH_LIMIT: usize,
    pub scene: OnceLock<sceneBVH>,
}

impl Global {
    pub fn init(){
        let global = Global { 
            global_object_id: AtomicU32::new(0), 
            global_object_list: OnceLock::new(),
            BVH_DEPTH_LIMIT: 20,
            scene: OnceLock::new(),
        };
        
        let r1: Result<(), Global> = GLOBAL.set(global);
        if r1.is_err() {
            panic!("Global instance already initialized");
        }
    }

    pub fn next_object_id(&self) -> u32 {
        println!("consumed id {:?}", self.global_object_id);
        self.global_object_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_objects(&self, objects: Vec<Shape>) -> Result<(), Vec<Shape>> {
        self.global_object_list.set(objects)
    }

    pub fn get_objects(&self) -> Option<&Vec<Shape>> {
        self.global_object_list.get()
    }

    /// Look up a shape by its registered ID.
    /// IDs are **not** guaranteed to equal the Vec index (mesh triangles consume
    /// IDs during construction before the Shape list is built), so we do a
    /// linear scan.  The scene object list is tiny (typically < 100 entries) so
    /// this is negligible compared to ray-intersection work.
    #[inline]
    pub fn get_object_by_id(&self, id: u32) -> Option<&Shape> {
        self.global_object_list
            .get()?
            .iter()
            .find(|s| s.get_id() == id)
    }

    #[inline(always)]
    pub fn get_BVH_depth_limit(&self) -> usize {
        self.BVH_DEPTH_LIMIT
    }

    #[inline(always)]
    pub fn get_scene(&self) -> &sceneBVH {
        self.scene.get().expect("Scene not initialized")
    }

}

// Global instance accessor
lazy_static::lazy_static! {
    pub static ref GLOBAL: OnceLock<Global> = OnceLock::new();
}

pub fn get_GLOBAL() -> &'static Global {
    GLOBAL.get().expect("Global instance not initialized")
}

// =============================================================================
// rotation axis and transform utils 
// =============================================================================

pub enum Axis {
    X, 
    Y,
    Z,
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub scale: f32,
}

impl Transform {
    pub fn new(position: Vec3, scale: f32) -> Self {
        Self { position, scale }
    }

    pub fn identity() -> Self {
        Self { position: Vec3::new(0.0, 0.0, 0.0), scale: 1.0 }
    }

    // -------------------------------------------------------------------------
    // Forward transforms  (local → world)
    // -------------------------------------------------------------------------

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        p * self.scale + self.position
    }

    pub fn transform_normal(&self, n: Vec3) -> Vec3 {
        // For a uniform scale the normal transform is the same as the point
        // transform (no translation, divide by scale² cancels to 1/scale which
        // re-normalises anyway).  We just need to renormalise after.
        (n / self.scale).normalize()
    }

    // -------------------------------------------------------------------------
    // Inverse transforms  (world → local)
    // -------------------------------------------------------------------------

    pub fn inverse_transform_point(&self, p: Vec3) -> Vec3 {
        (p - self.position) / self.scale
    }

    pub fn inverse_transform_direction(&self, d: Vec3) -> Vec3 {
        // Directions are not translated; only scale applies.
        d / self.scale
    }
}


//////////////////////////////////////////////////////
/// bounding box
/////////////////////////////////////////////////////


pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        BoundingBox { min, max }
    }

    pub fn new_empty() -> Self {
        BoundingBox {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// Slab-method AABB / ray intersection test.
    ///
    /// Returns `Some(tmin)` — the parametric entry distance — when the ray hits
    /// this box at any positive t, or `None` on a miss / fully-behind-origin box.
    /// The returned `tmin` can be used to sort child nodes by distance for
    /// nearest-first BVH traversal.
    pub fn hit(&self, ray: &Ray) -> Option<f32> {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        let mut t0 = (self.min.x - ray.origin.x) * ray.inv_dir.x;
        let mut t1 = (self.max.x - ray.origin.x) * ray.inv_dir.x;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin { return None; }

        let mut t0 = (self.min.y - ray.origin.y) * ray.inv_dir.y;
        let mut t1 = (self.max.y - ray.origin.y) * ray.inv_dir.y;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin { return None; }

        let mut t0 = (self.min.z - ray.origin.z) * ray.inv_dir.z;
        let mut t1 = (self.max.z - ray.origin.z) * ray.inv_dir.z;
        if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }
        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin { return None; }

        // Box is behind the ray origin
        if tmax < 0.0 { return None; }

        Some(tmin.max(0.0))
    }

    pub fn grow_to_fit(&mut self, new_shape_id: u32) {
        if let Some(shape) = get_GLOBAL().get_object_by_id(new_shape_id) {
            let shape_min = shape.get_min_bounds();
            let shape_max = shape.get_max_bounds();
            if self.min.x > shape_min.x { self.min.x = shape_min.x; }
            if self.min.y > shape_min.y { self.min.y = shape_min.y; }
            if self.min.z > shape_min.z { self.min.z = shape_min.z; }
            if self.max.x < shape_max.x { self.max.x = shape_max.x; }
            if self.max.y < shape_max.y { self.max.y = shape_max.y; }
            if self.max.z < shape_max.z { self.max.z = shape_max.z; }
        }
    }

    pub fn grow_to_fit_triangle(&mut self, tri: &Triangle) {
        let min_x = tri.p1.x.min(tri.p2.x).min(tri.p3.x);
        let min_y = tri.p1.y.min(tri.p2.y).min(tri.p3.y);
        let min_z = tri.p1.z.min(tri.p2.z).min(tri.p3.z);
        let max_x = tri.p1.x.max(tri.p2.x).max(tri.p3.x);
        let max_y = tri.p1.y.max(tri.p2.y).max(tri.p3.y);
        let max_z = tri.p1.z.max(tri.p2.z).max(tri.p3.z);
        if self.min.x > min_x { self.min.x = min_x; }
        if self.min.y > min_y { self.min.y = min_y; }
        if self.min.z > min_z { self.min.z = min_z; }
        if self.max.x < max_x { self.max.x = max_x; }
        if self.max.y < max_y { self.max.y = max_y; }
        if self.max.z < max_z { self.max.z = max_z; }
    }

}