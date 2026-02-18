use crate::vector::Vec3;

#[derive(Clone, Copy)]
pub struct Material {
    pub emmisive: bool,              // Whether the material is emmisive or not
    pub roughness: Option<f64>,           // 0.0 (smooth) to 1.0 (rough)
    pub color: Option<Vec3>,           // RGB color
    pub particle_density: Option<f64>,    // Density of particles in the material
    pub opaqueness: Option<f64>,          // 0.0 (transparent) to 1.0 (opaque)
}

impl Material {
    pub fn new(
        emmisive: bool,
        roughness: f64,
        color: Vec3,
        particle_density: f64,
        opaqueness: f64,
    ) -> Self {
        Material {
            emmisive,
            roughness: Some(roughness),
            color: Some(color),
            particle_density: Some(particle_density),
            opaqueness: Some(opaqueness),
        }
    }
}
