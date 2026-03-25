//! 3D Vector mathematics module for ray tracing operations.
//!
//! Provides a complete Vec3 implementation with all necessary operations
//! for ray tracing including dot products, cross products, normalization,
//! and random vector generation for Monte Carlo sampling.

use crate::utils::{Axis, random_double, random_double_range};

/// A 3D vector with x, y, z components.
///
/// This is the fundamental mathematical primitive used throughout the ray tracer
/// for representing positions, directions, colors, and normals.
#[derive(Copy, Clone, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Creates a new Vec3 with the given components.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Vector addition: returns self + other.
    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    /// Vector subtraction: returns self - other.
    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    /// Scalar multiplication: returns self * scalar.
    pub fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    /// Scalar division: returns self / scalar.
    pub fn div(self, s: f32) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s)
    }

    /// Component-wise multiplication: returns (self.x * other.x, self.y * other.y, self.z * other.z).
    /// Used for color blending and material attenuation.
    pub fn component_mul(self, o: Self) -> Self {
        Self::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }

    /// Dot product: returns self · other.
    /// Essential for lighting calculations, reflection, and angle computations.
    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Cross product: returns self × other.
    /// Used for calculating surface normals and coordinate system transformations.
    pub fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// Returns the squared magnitude of the vector.
    /// More efficient than length() when you only need to compare lengths.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the magnitude (length) of the vector.
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns a unit vector in the same direction.
    /// Essential for surface normals and ray directions.
    pub fn normalize(self) -> Self {
        let len = self.length();
        self / len
    }

    /// Clamps all components to the range [0, 1].
    /// Used for ensuring color values stay within valid bounds.
    pub fn clamp_vals_0_1(self) -> Self {
        fn c(v: f32) -> f32 {
            if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v }
        }
        Vec3::new(c(self.x), c(self.y), c(self.z))
    }

    /// Generates a random vector on the hemisphere defined by the given normal.
    /// Used for diffuse material scattering in Monte Carlo ray tracing.
    pub fn random_vec_on_hemisphere(normal: Vec3) -> Vec3 {
        let on_unit_sphere = Vec3::random_unit_vector();
        // Ensure the vector is in the same hemisphere as the normal
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }

    ///Generates a random vector on the unit sphere wheighted by the cosine of the angle to the normal.
    pub fn random_vec_cosine_weighted(&self, normal: &Vec3) -> Vec3 {
        let r1: f32 = random_double();
        let r2: f32 = random_double();
        let z = (1.0 - r2).sqrt();

        let phi = 2.0 * std::f32::consts::PI * r1;
        let x = phi.cos() * r2.sqrt();
        let y = phi.sin() * r2.sqrt();

        // Create an orthonormal basis (u, v, w) with w = normal
        let w = normal.normalize();
        let a = if w.x.abs() > 0.9 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
        let v = w.cross(a).normalize();
        let u = w.cross(v);

        // Convert from local space to world space
        u * x + v * y + w * z
    }

    /// Generates a random vector with components in [0, 1].
    pub fn random_vector() -> Vec3 {
        Vec3::new(random_double(), random_double(), random_double())
    }

    /// Generates a random vector with components in [min, max].
    pub fn random_vector_ranged(min: f32, max: f32) -> Vec3 {
        Vec3::new(
            random_double_range(min, max),
            random_double_range(min, max),
            random_double_range(min, max)
        )
    }

    pub fn random_unit_vector() -> Vec3 {
        loop {
            // Sample from [-1,1]³ instead of [0,1]³ for better distribution
            let randvec = Vec3::random_vector_ranged(-1.0, 1.0) ;
            let len_squared = randvec.length_squared();
            // Reject vectors that are too small (numerical instability) or too large
            if len_squared > 1e-16 && len_squared <= 1.0 {
                return randvec / len_squared.sqrt();
            }
        }
    }

    pub fn random_vec_on_circle() -> Vec3{
        loop {
            let vec = Vec3::new(random_double(), random_double(), 0.0);
            // you can use length sqared instead of length because it saves a sqrt 
            //while keeping the property whether the vec's length is greater or less than one
            if vec.length_squared() <= 1.0 {
                return vec;
            }
        }
    }

    pub fn near_zero(v: Vec3) -> bool {
        const EPSILON: f32 = 1e-8;
        v.x.abs() < EPSILON && v.y.abs() < EPSILON && v.z.abs() < EPSILON
    }

    #[inline(always)]
    pub fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// Rotate this vector by `angle` radians around the given cardinal axis.
    ///
    /// Uses a standard 3×3 rotation matrix — no heap allocation, fully inlined.
    ///
    /// ```rust
    /// let v = Vec3::new(1.0, 0.0, 0.0);
    /// let rotated = v.rotate(std::f32::consts::FRAC_PI_2, Axis::Z);
    /// // ≈ (0, 1, 0)
    /// ```
    pub fn rotate(&self, angle: f32, position: Vec3, axis: Axis) -> Vec3 {
        let pos = *self - position;
        let (sin, cos) = angle.sin_cos();
        match axis {
            // Rotation matrix around X:
            // [ 1    0     0  ]
            // [ 0   cos  -sin ]
            // [ 0   sin   cos ]
            Axis::X => Vec3::new(
                pos.x,
                pos.y * cos - pos.z * sin,
                pos.y * sin + pos.z * cos,
            ) + pos,
            // Rotation matrix around Y:
            // [  cos  0  sin ]
            // [   0   1   0  ]
            // [ -sin  0  cos ]
            Axis::Y => Vec3::new(
                pos.x * cos + pos.z * sin,
                pos.y,
                -pos.x * sin + pos.z * cos,
            ) + pos,
            // Rotation matrix around Z:
            // [ cos  -sin  0 ]
            // [ sin   cos  0 ]
            // [  0     0   1 ]
            Axis::Z => Vec3::new(
                pos.x * cos - pos.y * sin,
                pos.x * sin + pos.y * cos,
                pos.z,
            ) + pos,
        }
    }

    /// Rotate this vector by `angle` radians around an **arbitrary axis**
    /// using Rodrigues' rotation formula:
    ///
    ///   v_rot = v·cos(θ) + (k × v)·sin(θ) + k·(k·v)·(1 − cos(θ))
    ///
    /// `axis_vec` does **not** need to be pre-normalized — this method
    /// normalizes it internally.
    ///
    /// ```rust
    /// // Tilt a direction 45° around the world Y axis
    /// let dir = Vec3::new(0.0, 0.0, -1.0);
    /// let tilted = dir.rotate_around(std::f32::consts::FRAC_PI_4, Vec3::new(0.0, 1.0, 0.0));
    /// ```
    pub fn rotate_around(&self, angle: f32, axis_vec: Vec3) -> Vec3 {
        let k = axis_vec.normalize();           // unit axis
        let (sin, cos) = angle.sin_cos();
        let dot = k.dot(*self);

        // Rodrigues: v·cos θ  +  (k × v)·sin θ  +  k·(k·v)·(1 − cos θ)
        *self * cos + k.cross(*self) * sin + k * (dot * (1.0 - cos))
    }

}

// Operator overloads for more natural mathematical syntax
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign};

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 { self.add(o) }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 { self.sub(o) }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f32) -> Vec3 { self.mul(s) }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f32) -> Vec3 { self.div(s) }
}

impl Div<Vec3> for f32 {
    type Output = Vec3;
    fn div(self, o: Vec3) -> Vec3 {o * (1.0 / self)}
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 { Vec3::new(-self.x, -self.y, -self.z) }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, o: Self) { *self = *self + o }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, o: Self) { *self = *self - o }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, s: Vec3) { *self = self.component_mul(s) }
}

/// Allows f32 * Vec3 syntax (in addition to Vec3 * f32)
impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, o: Vec3) -> Vec3 { o.mul(self) }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, o: Vec3) -> Vec3 { self.component_mul(o) }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}