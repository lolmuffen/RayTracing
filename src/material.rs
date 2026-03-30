use crate::intersection::Intersection;
use crate::vector::Vec3;
use crate::ray::Ray;
use crate::utils::random_double;

/// Consolidated material kind. Each variant holds its parameters.
/// Methods are implemented directly on the enum — no trait indirection,
/// no heap allocation, no vtable. scatter() can now be fully inlined
/// by the compiler in the hot ray-bounce path.
#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertian { albedo: f32, color: Vec3 },
    Metal { albedo: f32, color: Vec3, roughness: f32 },
    Glass { refraction_index: f32, albedo: f32, color: Vec3 },
    Volume { density: f32, color: Vec3 },
    Emissive { color: Vec3, intensity: f32 },
    Specular { specular_probability: f32, color: Vec3, albedo: f32, roughness: f32 },
}

impl Material {
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    pub fn lambertian(albedo: f32, color: Vec3) -> Self {
        Self::Lambertian { albedo, color }
    }

    pub fn metal(albedo: f32, color: Vec3, roughness: f32) -> Self {
        Self::Metal { albedo, color, roughness: roughness.clamp(0.0, 1.0) }
    }

    pub fn glass(refraction_index: f32, albedo: f32, color: Vec3) -> Self {
        Self::Glass { refraction_index, albedo, color }
    }

    pub fn volume(density: f32, color: Vec3) -> Self {
        Self::Volume { density, color }
    }

    pub fn emissive(color: Vec3, intensity: f32) -> Self {
        Self::Emissive { color, intensity }
    }

    pub fn specular(color: Vec3, albedo: f32, roughness: f32, specular_probability: f32) -> Self {
        Self::Specular {
            specular_probability,
            color,
            albedo,
            roughness: roughness.clamp(0.0, 1.0),
        }
    }

    // -------------------------------------------------------------------------
    // Core material interface — previously behind `dyn Material` vtable
    // -------------------------------------------------------------------------

    pub fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Ray {
        let hitdata = hit_rec
            .hitdata
            .as_ref()
            .expect("scatter() called with no hit data");

        let attenuation_factor = 1.0 / (1.0 + hitdata.distance);

        match *self {
            Material::Lambertian { albedo, color } => {
                let scatter_direction =
                    hitdata.normal + ray_in.direction.random_vec_cosine_weighted(&hitdata.normal);
                Ray::new(
                    hitdata.hit_point,
                    scatter_direction.normalize(),
                    ray_in.color * color * albedo * attenuation_factor,
                )
            }

            Material::Metal { albedo, color, roughness } => {
                let reflected = Self::reflect(ray_in.direction.normalize(), hitdata.normal);
                let roughness_offset =
                    ray_in.direction.random_vec_cosine_weighted(&hitdata.normal) * roughness;
                let scattered_dir = (reflected + roughness_offset).normalize();
                Ray::new(
                    hitdata.hit_point,
                    scattered_dir,
                    ray_in.color * color * albedo * attenuation_factor,
                )
            }

            Material::Glass { refraction_index, albedo, color } => {
                let outward_normal = if ray_in.direction.dot(hitdata.normal) > 0.0 {
                    -hitdata.normal
                } else {
                    hitdata.normal
                };
                let ni_over_nt = if ray_in.direction.dot(hitdata.normal) > 0.0 {
                    refraction_index
                } else {
                    1.0 / refraction_index
                };
                let cos_theta = (-ray_in.direction.normalize()).dot(outward_normal).min(1.0);
                let refracted = Self::refract(ray_in.direction, outward_normal, ni_over_nt);
                let reflect_prob = if refracted
                    .is_some() { Self::schlick_reflectance(cos_theta, refraction_index) } else { 1.0 };

                let direction = if random_double() < reflect_prob {
                    Self::reflect(ray_in.direction, outward_normal)
                } else {
                    refracted
                        .unwrap_or_else(|| Self::reflect(ray_in.direction, outward_normal))
                };
                Ray::new(
                    hitdata.hit_point,
                    direction.normalize(),
                    ray_in.color * albedo * color * attenuation_factor,
                )
            }

            Material::Volume { density, color } => {
                let attenuation = ray_in.color * color * density * attenuation_factor;
                let scatter_direction = hitdata.normal + Vec3::random_unit_vector();
                Ray::new(
                    hitdata.hit_point,
                    scatter_direction.normalize(),
                    attenuation,
                )
            }

            Material::Emissive { color, intensity } => Ray::new(
                ray_in.origin,
                ray_in.direction,
                ray_in.color * color * intensity * attenuation_factor,
            ),

            Material::Specular { specular_probability, color, albedo, roughness } => {
                let specular_reflection = specular_probability >= random_double();
                let (scatter_direction, out_color) = if specular_reflection {
                    let dir =
                        hitdata.normal + ray_in.direction.random_vec_cosine_weighted(&hitdata.normal);
                    (dir, ray_in.color * color * albedo * attenuation_factor)
                } else {
                    let reflected = Self::reflect(ray_in.direction.normalize(), hitdata.normal);
                    let roughness_offset =
                        ray_in.direction.random_vec_cosine_weighted(&hitdata.normal) * roughness;
                    let dir = (reflected + roughness_offset).normalize();
                    (dir, ray_in.color * albedo * attenuation_factor)
                };
                Ray::new(hitdata.hit_point, scatter_direction.normalize(), out_color)
            }
        }
    }

    pub fn emitted(&self) -> Vec3 {
        if let Material::Emissive { color, intensity } = *self {
            color * intensity
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        }
    }

    pub fn is_emissive(&self) -> bool {
        matches!(self, Material::Emissive { .. })
    }

    // -------------------------------------------------------------------------
    // Private helpers (previously on Generic)
    // -------------------------------------------------------------------------

    fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - n * 2.0 * v.dot(n)
    }

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

}