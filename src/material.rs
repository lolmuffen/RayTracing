use crate::vector::Vec3;

#[derive(Clone, Copy)]
pub struct Material {
    pub roughness: f64,           // 0.0 (smooth) to 1.0 (rough)
    pub color: Vec3,           // RGB color
    pub emissiveness: f64,        // 0.0 (non-emissive) to 1.0 (fully emissive)
    pub particle_density: f64,    // Density of particles in the material
    pub opaqueness: f64,          // 0.0 (transparent) to 1.0 (opaque)
}

impl Material {
    pub fn new(
        roughness: f64,
        color: Vec3,
        emissiveness: f64,
        particle_density: f64,
        opaqueness: f64,
    ) -> Self {
        Material {
            roughness,
            color,
            emissiveness,
            particle_density,
            opaqueness,
        }
    }
}
