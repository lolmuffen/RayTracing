use crate::intersection::{Hit, Intersection};
use crate::vector::Vec3;
use crate::ray::Ray;
use crate::utils::random_double;

/// Material trait for defining how surfaces interact with light
pub trait Material: Send + Sync {
    /// Scatter a ray off this material
    /// Returns Option containing a scattered `Ray` (attenuation encoded in `Ray.color`)
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray>;

    /// Optional emitted light from this material
    fn emitted(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    fn is_emissive(&self) -> bool {
        self.emitted() != Vec3::new(0.0, 0.0, 0.0)
    }
}

/// Consolidated material kind. Each variant holds its parameters.
#[derive(Clone, Copy, Debug)]
pub enum MaterialKind {
    Lambertian { albedo: f32, color: Vec3 },
    Metal { albedo: f32, color: Vec3, roughness: f32 },
    Glass { refraction_index: f32, albedo: f32, color: Vec3 },
    Volume { density: f32, color: Vec3 },
    Emissive { color: Vec3, intensity: f32 },
    Specular { specular_probability: f32, color: Vec3, albedo: f32, roughness: f32 },
    Default,
}

/// Unified `Material` type that dispatches scatter behavior based on `MaterialKind`.
pub struct Generic {
    pub kind: MaterialKind,
}

impl Generic {
    pub fn new(kind: MaterialKind) -> Self {
        Self { kind }
    }

    pub fn lambertian(albedo: f32, color: Vec3) -> Self {
        Self { kind: MaterialKind::Lambertian { albedo, color } }
    }

    pub fn metal(albedo: f32, color: Vec3, roughness: f32) -> Self {
        Self { kind: MaterialKind::Metal { albedo, color, roughness: roughness.clamp(0.0, 1.0) } }
    }

    pub fn glass(refraction_index: f32, albedo: f32, color: Vec3) -> Self {
        Self { kind: MaterialKind::Glass { refraction_index, albedo, color } }
    }

    pub fn volume(density: f32, color: Vec3) -> Self {
        Self { kind: MaterialKind::Volume { density, color } }
    }

    pub fn emissive(color: Vec3, intensity: f32) -> Self {
        Self { kind: MaterialKind::Emissive { color, intensity } }
    }

    pub fn specular(color: Vec3, albedo: f32, roughness: f32, specular_probability: f32) -> Self {
        Self { kind: MaterialKind::Specular { specular_probability, color, albedo, roughness: roughness.clamp(0.0, 1.0) } }
    }

    // Helper: reflect a vector around a normal
    fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - n * 2.0 * v.dot(n)
    }

    // Helper: refract vector with index ratio
    fn refract(v: Vec3, n: Vec3, ni_over_nt: f32) -> Option<Vec3> {
        let uv = v.normalize();
        let dt = uv.dot(n);
        let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
        if discriminant > 0.0 {
            let refracted = (uv - n * dt) * ni_over_nt - n * discriminant.sqrt();
            Some(refracted)
        } else {
            None
        }
    }

    fn schlick_reflectance(cos_theta: f32, refraction_index: f32) -> f32 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
    }

    // Unified scatter function for all reflective materials
    fn scatter_reflective(&self, ray_in: &Ray, hitdata: &Hit) -> Option<Ray> {
        let attenuation_factor = 1.0 / (1.0 + hitdata.distance);

        match self.kind {
            MaterialKind::Lambertian { albedo, color } => {
                let scatter_direction = hitdata.normal + ray_in.direction.random_vec_cosine_weighted(&hitdata.normal);
                Some(Ray::new(hitdata.hit_point, scatter_direction.normalize(), ray_in.color * color * albedo * attenuation_factor))
            }
            MaterialKind::Metal { albedo, color, roughness } => {
                let reflected = Generic::reflect(ray_in.direction.normalize(), hitdata.normal);
                let roughness_offset = ray_in.direction.random_vec_cosine_weighted(&hitdata.normal) * roughness;
                let scattered_dir = (reflected + roughness_offset).normalize();
                Some(Ray::new(hitdata.hit_point, scattered_dir, ray_in.color * color * albedo * attenuation_factor))
            }
            MaterialKind::Glass { refraction_index, albedo, color } => {
                let outward_normal = if ray_in.direction.dot(hitdata.normal) > 0.0 { -hitdata.normal } else { hitdata.normal };
                let ni_over_nt = if ray_in.direction.dot(hitdata.normal) > 0.0 { refraction_index } else { 1.0 / refraction_index };
                let cos_theta = (-ray_in.direction.normalize()).dot(outward_normal).min(1.0);
                let refracted = Generic::refract(ray_in.direction, outward_normal, ni_over_nt);
                let reflect_prob = refracted.is_some().then(|| Generic::schlick_reflectance(cos_theta, refraction_index)).unwrap_or(1.0);
                
                let direction = if random_double() < reflect_prob {
                    Generic::reflect(ray_in.direction, outward_normal)
                } else {
                    refracted.unwrap_or_else(|| Generic::reflect(ray_in.direction, outward_normal))
                };
                Some(Ray::new(hitdata.hit_point, direction.normalize(), ray_in.color * albedo * color * attenuation_factor))
            }
            MaterialKind::Specular { specular_probability, color, albedo, roughness } => {
                let specular_reflection = specular_probability >= random_double();
                let (scatter_direction, out_color) = if specular_reflection {
                    let dir = hitdata.normal + ray_in.direction.random_vec_cosine_weighted(&hitdata.normal);
                    (dir, ray_in.color * color * albedo * attenuation_factor)
                } else {
                    let reflected = Generic::reflect(ray_in.direction.normalize(), hitdata.normal);
                    let roughness_offset = ray_in.direction.random_vec_cosine_weighted(&hitdata.normal) * roughness;
                    let dir = (reflected + roughness_offset).normalize();
                    (dir, ray_in.color * albedo * attenuation_factor)
                };
                Some(Ray::new(hitdata.hit_point, scatter_direction.normalize(), out_color))
            }
            _ => None,
        }
    }
}

impl Material for Generic {
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray> {
        let hitdata = match hit_rec.hitdata.clone() {
            Some(data) => data,
            None => return None,
        };

        match self.kind {
            MaterialKind::Volume { .. } | MaterialKind::Emissive { .. } => {
                // Volume and Emissive materials handled separately
                match self.kind {
                    MaterialKind::Volume { density, color } => {
                        let attenuation = ray_in.color * color * density * (1.0 / (1.0 + hitdata.distance));
                        let scatter_direction = hitdata.normal + Vec3::random_unit_vector();
                        Some(Ray::new(hitdata.hit_point, scatter_direction.normalize(), attenuation))
                    }
                    MaterialKind::Emissive { color, intensity } => {
                        Some(Ray::new(ray_in.origin, ray_in.direction, ray_in.color * color * intensity * (1.0 / (1.0 + hitdata.distance))))
                    }
                    _ => None,
                }
            }
            _ => self.scatter_reflective(ray_in, &hitdata),
        }
    }

    fn emitted(&self) -> Vec3 {
        if let MaterialKind::Emissive { color, intensity } = self.kind {
            color * intensity
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        }
    }
}
