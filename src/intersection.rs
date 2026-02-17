
use crate::vector::Vec3;

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

pub struct Intersection {
    pub hit: bool,
    pub hits: Option<Vec<Hit>>,
    pub object_id: Option<u32>,
}

impl Intersection {
    pub fn new(hit: bool, hits: Option<Vec<Hit>>, object_id: Option<u32>) -> Intersection {
        Intersection { hit, hits, object_id }
    }
}