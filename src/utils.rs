//! utils.rs
//! Mathematical utilities and helper functions for ray tracing operations.
//!
//! This module provides essential mathematical building blocks used throughout
//! the ray tracer including random number generation, coordinate transformations,
//! and interval arithmetic for robust intersection testing.

use rand::RngExt;
use crate::{shape::Shape, vector::Vec3};
use std::sync::{Arc, Mutex, OnceLock};
use crate::BVH::sceneBVH;


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
    pub global_object_id: Arc<Mutex<u32>>,
    pub global_object_list: OnceLock<Vec<Box<dyn Shape + Send + Sync>>>,
    pub BVH_DEPTH_LIMIT: usize,
    pub Scene: Arc<Mutex<sceneBVH>>
}

impl Global {
    pub fn new() -> Global {
        Global { 
            global_object_id: Arc::new(Mutex::new(0)), 
            global_object_list: OnceLock::new(),
            BVH_DEPTH_LIMIT: 20, // Default depth limit for BVH tree
            Scene: Arc::new(Mutex::new(sceneBVH::new())), // Initialize empty BVH tree
        }
    }

    pub fn next_object_id(&self) -> u32 {
        let mut guard = self.global_object_id.lock().unwrap();
        *guard += 1;
        *guard
    }

    pub fn set_objects(&self, objects: Vec<Box<dyn Shape + Send + Sync>>) -> Result<(), Vec<Box<dyn Shape + Send + Sync>>> {
        self.global_object_list.set(objects)
    }

    pub fn get_objects(&self) -> Option<&Vec<Box<dyn Shape + Send + Sync>>> {
        self.global_object_list.get()
    }

    pub fn get_object_by_id(&self, id: u32) -> Option<&Box<dyn Shape + Send + Sync>> {
        self.global_object_list.get()?.iter().find(|obj| obj.get_id() == id)
    }

}

// Global instance accessor
lazy_static::lazy_static! {
    pub static ref GLOBAL: Global = Global::new();
}