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
    if u < 0.0 || u > 1.0 {
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
    return x;
}

pub fn random_double_range(min: f32, max: f32) -> f32 {
    let mut rng = rand::rng();
    let y: f32 = rng.random_range(min..max);
    return y;
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
// rotation axis utils
// =============================================================================

pub enum Axis {
    X, 
    Y,
    Z,
}