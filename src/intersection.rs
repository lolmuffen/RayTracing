use crate::vector::Vec3;
use crate::material::Material;

#[derive(Debug, Clone)]
pub struct Hit {
    pub distance: f32,
    pub hit_point: Vec3,
    pub normal: Vec3,
    pub material: Option<Material>,
}

impl Hit {
    pub fn new(distance: f32, hit_point: Vec3, normal: Vec3) -> Hit {
        Hit { distance, hit_point, normal, material: None }
    }

    pub fn new_with_material(distance: f32, hit_point: Vec3, normal: Vec3, material: Material) -> Hit {
        Hit { distance, hit_point, normal, material: Some(material) }
    }
}

#[derive(Debug, Clone)]
pub struct Intersection {
    pub hit: bool,
    pub hitdata: Option<Hit>,
    pub object_id: Option<u32>,
}

impl Intersection {
    pub fn new(hit: bool, hitdata: Option<Hit>, object_id: Option<u32>) -> Intersection {
        Intersection { hit, hitdata, object_id }
    }
}