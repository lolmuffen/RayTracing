
use crate::vector::Vec3;

#[derive(Debug, Clone)]
pub struct Hit {
    pub distance: f32,           
    pub hit_point: Vec3,        
    pub normal: Vec3,
}

impl Hit {
    pub fn new(distance: f32, hit_point: Vec3, normal: Vec3) -> Hit {
        Hit { distance, hit_point, normal }
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