use crate::intersection::Intersection;
use crate::vector::Vec3;
use crate::ray::Ray;
use crate::utils::random_double;

/// Material trait for defining how surfaces interact with light
pub trait Material: Send + Sync {
    /// Scatter a ray off this material
    /// Returns Option containing:
    /// - attenuation (color): how much light is absorbed/reflected
    /// - scattered_ray: the resulting ray direction after interaction
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray>;
    
    /// Optional emitted light from this material
    fn emitted(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    fn is_emissive(&self) -> bool {
        self.emitted() != Vec3::new(0.0, 0.0, 0.0)
    }
}

/// Lambertian (matte/diffuse) material - reflects light uniformly in all directions
pub struct Lambertian {
    pub albedo: f32, // Base color/reflectivity
    pub color: Vec3, // Color of the material
}

impl Lambertian {
    pub fn new(albedo: f32, color: Vec3) -> Self {
        Lambertian { albedo, color }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray> {

        let hitdata = match hit_rec.hitdata.clone() {
            Some(data) => data,
            None => return None, // No hit data, cannot scatter
        };
        let scatter_direction = hitdata.normal + _ray_in.direction.random_vec_cosine_weighted(&hitdata.normal);
        
        let scattered = Ray::new(
            hitdata.hit_point,
            scatter_direction.normalize(),
            _ray_in.color * self.color * self.albedo * (1.0 / (1.0 + hitdata.distance)),
        );
        Some(scattered)
    }
}

/// Metal material - reflects light specularly with optional roughness
pub struct Metal {
    pub albedo: f32, // Reflectivity
    pub color: Vec3, // Color of the metal
    pub roughness: f32, // 0.0 (mirror-like) to 1.0 (rough)
}

impl Metal {
    pub fn new(albedo: f32, color: Vec3, roughness: f32) -> Self {
        Metal {
            albedo,
            color,
            roughness: roughness.clamp(0.0, 1.0),
        }
    }
    
    fn reflect(v: Vec3, n: Vec3) -> Vec3 {
        v - n * 2.0 * v.dot(n)
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray> {
        let hitdata = match hit_rec.hitdata.clone() {
            Some(data) => data,
            None => return None, // No hit data, cannot scatter
        };
        
        let reflected = Metal::reflect(ray_in.direction.normalize(), hitdata.normal);
        
        // Add roughness by perturbing the reflected direction
        let roughness_offset = ray_in.direction.random_vec_cosine_weighted(&hitdata.normal) * self.roughness;
        let scattered_dir = (reflected + roughness_offset).normalize();
        
        // Only scatter if the ray is going outward
        
        let scattered = Ray::new(hitdata.hit_point, scattered_dir, ray_in.color * self.color * self.albedo * (1.0 / (1.0 + hitdata.distance)));
        Some(scattered)
    }
}

/// Glass/Dielectric material - refracts light with Fresnel reflection
pub struct Glass {
    pub albedo: f32, // Reflectivity for the reflected component
    pub color: Vec3, // Color of the glass (usually white or slightly tinted)
    pub refraction_index: f32, // IOR: 1.5 for glass, 2.4 for diamond
}

impl Glass {
    pub fn new(refraction_index: f32, albedo: f32, color: Vec3) -> Self {
        Glass {
            albedo,
            color,
            refraction_index,
        }
    }
    
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

impl Material for Glass {
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray> {
        let hitdata = match hit_rec.hitdata.clone() {
            Some(data) => data,
            None => return None, // No hit data, cannot scatter
        };
        let outward_normal = if ray_in.direction.dot(hitdata.normal) > 0.0 {
            -hitdata.normal
        } else {
            hitdata.normal
        };
        
        let ni_over_nt = if ray_in.direction.dot(hitdata.normal) > 0.0 {
            self.refraction_index
        } else {
            1.0 / self.refraction_index
        };
        
        let cos_theta = (-ray_in.direction.normalize()).dot(outward_normal).min(1.0);
        
        let refracted = Glass::refract(ray_in.direction, outward_normal, ni_over_nt);
        
        let reflect_prob = if let Some(_) = refracted {
            Glass::schlick_reflectance(cos_theta, self.refraction_index)
        } else {
            1.0
        };
        
        let direction = if random_double() < reflect_prob {
            Glass::reflect(ray_in.direction, outward_normal)
        } else if let Some(refr) = refracted {
            refr
        } else {
            Glass::reflect(ray_in.direction, outward_normal)
        };
        
        let scattered = Ray::new(hitdata.hit_point, direction.normalize(), ray_in.color * self.albedo * self.color * (1.0 / (1.0 + hitdata.distance)));
        Some(scattered)
    }
}

pub struct Volume {
    pub density: f32,
    pub color: Vec3,
}

impl Volume {
    pub fn new(density: f32, color: Vec3) -> Self {
        Volume { density, color }
    }
    
}

impl Material for Volume {
    fn scatter(&self, ray_in: &Ray, hit_rec: &Intersection) -> Option<Ray> {
        let hitdata = match hit_rec.hitdata.clone() {
            Some(data) => data,
            None => return None, // No hit data, cannot scatter
        };
        // Simple volumetric scattering - attenuate the ray and scatter in a random direction
        let attenuation = ray_in.color * self.color * self.density * (1.0 / (1.0 + hitdata.distance)); // Attenuation based on density and distance traveled
        let scatter_direction = hitdata.normal + Vec3::random_unit_vector();
        let scattered = Ray::new(hitdata.hit_point, scatter_direction.normalize(), attenuation);
        Some(scattered)
    }
}

pub struct Default;

impl Default {
    pub fn new() -> Self {
        Default
    }
}

impl Material for Default {
    fn scatter(&self, _ray_in: &Ray, _hit_rec: &Intersection) -> Option<Ray> {
        None // No scattering, fully absorbs light
    }
}


pub struct Emissive {
    pub color: Vec3,
    pub intensity: f32,
}

impl Emissive {
    pub fn new(color: Vec3, intensity: f32) -> Self {
        Emissive { color, intensity }
    }
}

impl Material for Emissive {
    fn scatter(&self, ray_in: &Ray, _hit_rec: &Intersection) -> Option<Ray> {
        return Some(Ray::new(ray_in.origin, ray_in.direction, ray_in.color * self.color * self.intensity * (1.0 / (1.0 + _hit_rec.hitdata.as_ref().unwrap().distance)))); // Emissive materials do not scatter light
    }
    fn emitted(&self) -> Vec3 {
        self.color * self.intensity
    }
}