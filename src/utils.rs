//! utils.rs
//! Mathematical utilities and helper functions for ray tracing operations.
//!
//! This module provides essential mathematical building blocks used throughout
//! the ray tracer including random number generation, coordinate transformations,
//! and interval arithmetic for robust intersection testing.

use rand::RngExt;
use crate::main;
use crate::{shape::Shape, vector::Vec3};
use std::sync::{Arc, Mutex, OnceLock};
use crate::BVH::sceneBVH;
use crate::light::Light;


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
    pub global_light_id: Arc<Mutex<u32>>,
    pub global_light_list: OnceLock<Vec<Box<dyn Light + Send + Sync>>>,
    pub BVH_DEPTH_LIMIT: usize,
    pub scene: OnceLock<sceneBVH>,
    pub bounce_depth_limit: Arc<Mutex<u32>>,
}

impl Global {
    pub fn init(){
        let global = Global { 
            global_object_id: Arc::new(Mutex::new(0)), 
            global_object_list: OnceLock::new(),
            global_light_id: Arc::new(Mutex::new(0)),
            global_light_list: OnceLock::new(),
            BVH_DEPTH_LIMIT: 20, // Default depth limit for BVH tree
            scene: OnceLock::new(), // Initialize empty BVH tree
            bounce_depth_limit: Arc::new(Mutex::new(16)), // Initialize empty bounce depth limit
        };
        
        let r1: Result<(), Global> = GLOBAL.set(global); // Set the global instance
        if r1.is_err() {
            panic!("Global instance already initialized");
        }
    }

    pub fn next_object_id(&self) -> u32 {
        let mut guard = self.global_object_id.lock().unwrap();
        *guard += 1;
        *guard
    }

    pub fn next_light_id(&self) -> u32 {
        let mut guard = self.global_light_id.lock().unwrap();
        *guard += 1;
        *guard
    }

    pub fn set_objects(&self, objects: Vec<Box<dyn Shape + Send + Sync>>) -> Result<(), Vec<Box<dyn Shape + Send + Sync>>> {
        self.global_object_list.set(objects)
    }

    pub fn set_lights(&self, lights: Vec<Box<dyn Light + Send + Sync>>) -> Result<(), Vec<Box<dyn Light + Send + Sync>>> {
        self.global_light_list.set(lights)
    }

    pub fn get_objects(&self) -> Option<&Vec<Box<dyn Shape + Send + Sync>>> {
        self.global_object_list.get()
    }

    pub fn get_object_by_id(&self, id: u32) -> Option<&Box<dyn Shape + Send + Sync>> {
        self.global_object_list.get()?.iter().find(|obj| obj.get_id() == id)
    }

    pub fn get_light_by_id(&self, id: u32) -> Option<&Box<dyn Light + Send + Sync>> {
        self.global_light_list.get()?.iter().find(|light| light.get_id() == id)
    }

    pub fn get_depth_limit(&self) -> u32 {
        let guard = self.bounce_depth_limit.lock().unwrap();
        *guard
    }

    pub fn get_scene(&self) -> &sceneBVH {
        self.scene.get().expect("Scene not initialized")
    }

    pub fn get_lights(&self) -> Option<&Vec<Box<dyn Light + Send + Sync>>> {
        self.global_light_list.get()
    }
}

// Global instance accessor
lazy_static::lazy_static! {
    pub static ref GLOBAL: OnceLock<Global> = OnceLock::new();
}

pub fn get_GLOBAL() -> &'static Global {
    GLOBAL.get().expect("Global instance not initialized")
}
