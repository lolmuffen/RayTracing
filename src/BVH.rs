use crate::intersection::{Hit, Intersection};
use crate::ray::Ray;
use crate::utils::{Global, get_GLOBAL};
use crate::vector::Vec3;
use crate::gpu_structs::GpuBvhNode;

pub struct sceneBVH {
    pub ID: u8, // 0 for root, 1 for left child, 2 for right child
    pub bounding_box: BoundingBox,
    pub left_child: Option<Box<sceneBVH>>,
    pub right_child: Option<Box<sceneBVH>>,
    pub shape_ids: Option<Vec<u32>>, // Leaf node contains shape IDs
}

impl sceneBVH {
    /// Create a new sceneBVH tree from all global objects
    pub fn new() -> Self {
        let objects = get_GLOBAL().get_objects().unwrap();
        let shape_ids: Vec<u32> = objects.iter().map(|obj| obj.get_id()).collect();
        
        sceneBVH::build_recursive(shape_ids, 0, 0)
    }

    /// Recursively build sceneBVH tree by splitting shapes in half
    fn build_recursive(shape_ids: Vec<u32>, depth: usize, child_ID: u8) -> Self {
        // Calculate bounding box for current set of shapes
        let mut bounding_box = BoundingBox::new_empty();



        for &id in &shape_ids {
            bounding_box.grow_to_fit(id);
        }

        if depth >= get_GLOBAL().get_BVH_depth_limit() as usize || shape_ids.len() <= 2 {
            // Create leaf node
            return sceneBVH {
                bounding_box,
                left_child: None,
                right_child: None,
                shape_ids: Some(shape_ids),
                ID: child_ID,
            };
        }

        // Compute centroid of the bounding box
        let centroid = (bounding_box.min + bounding_box.max) * 0.5;

        // Compute the principal axis (longest dimension of bounding box)
        let size = bounding_box.max - bounding_box.min;
        let principal_axis = if size.x >= size.y && size.x >= size.z {
            Vec3::new(1.0, 0.0, 0.0)
        } else if size.y >= size.z {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };

        // Sort shapes by their projection onto the principal axis
        let mut sorted_ids = shape_ids.clone();
        sorted_ids.sort_by(|&a, &b| {
            let center_a = if let Some(shape) = get_GLOBAL().get_object_by_id(a) {
                (shape.get_min_bounds() + shape.get_max_bounds()) * 0.5
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            };

            let center_b = if let Some(shape) = get_GLOBAL().get_object_by_id(b) {
                (shape.get_min_bounds() + shape.get_max_bounds()) * 0.5
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            };

            let proj_a = (center_a - centroid).dot(principal_axis);
            let proj_b = (center_b - centroid).dot(principal_axis);

            proj_a.partial_cmp(&proj_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Split at the median
        let mid = sorted_ids.len() / 2;
        let left_ids = sorted_ids[..mid].to_vec();
        let right_ids = sorted_ids[mid..].to_vec();

        let left_child = Box::new(sceneBVH::build_recursive(left_ids, depth + 1, 1));
        let right_child = Box::new(sceneBVH::build_recursive(right_ids, depth + 1, 2));

        sceneBVH {
            bounding_box,
            left_child: Some(left_child),
            right_child: Some(right_child),
            shape_ids: None,
            ID: child_ID,
        }

    }

    pub fn traverse(&self, ray: &Ray, global: &Global) -> Option<Intersection> {
        // AABB - ray intersection (slab method)
        let mut tmin = (self.bounding_box.min.x - ray.origin.x) * ray.inv_dir.x;
        let mut tmax = (self.bounding_box.max.x - ray.origin.x) * ray.inv_dir.x;
        if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

        let mut tymin = (self.bounding_box.min.y - ray.origin.y) * ray.inv_dir.y;
        let mut tymax = (self.bounding_box.max.y - ray.origin.y) * ray.inv_dir.y;
        if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }

        if tymin > tmin { tmin = tymin; }
        if tymax < tmax { tmax = tymax; }

        let mut tzmin = (self.bounding_box.min.z - ray.origin.z) * ray.inv_dir.z;
        let mut tzmax = (self.bounding_box.max.z - ray.origin.z) * ray.inv_dir.z;
        if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }

        if tzmin > tmin { tmin = tzmin; }
        if tzmax < tmax { tmax = tzmax; }

        if tmax < 0.0 {
            return None;
        }

        // If leaf node, test contained shapes
        if let Some(ids) = &self.shape_ids {
            let mut best_hit: Option<Hit> = None;
            let mut best_obj_id: Option<u32> = None;

            for &id in ids.iter() {
                if let Some(shape) = global.get_object_by_id(id) {
                    let inter = shape.intersect(ray);
                    if inter.hit {
                        if let Some(hits) = inter.hitdata {
                            let h = hits;
                            if h.distance > 0.001 {
                                let is_better = match &best_hit {
                                    Some(bh) => h.distance < bh.distance,
                                    None => true,
                                };
                                if is_better {
                                    best_hit = Some(Hit::new(h.distance, h.hit_point, h.normal));
                                    best_obj_id = inter.object_id;
                                }
                            }
                            
                        }
                    }
                }
            }

            if let Some(h) = best_hit {
                return Some(Intersection::new(true, Some(h), best_obj_id));
            }

            return None;
        }

        // Internal node: traverse children
        let mut left_hit: Option<Intersection> = None;
        let mut right_hit: Option<Intersection> = None;

        if let Some(left) = &self.left_child {
            left_hit = left.traverse(ray, &global);
        }

        if let Some(right) = &self.right_child {
            right_hit = right.traverse(ray, &global);
        }

        match (left_hit, right_hit) {
            (Some(l), Some(r)) => {
                // pick nearer of the two
                let ld = l.hitdata.as_ref().map(|h| h.distance).unwrap_or(f32::INFINITY);
                let rd = r.hitdata.as_ref().map(|h| h.distance).unwrap_or(f32::INFINITY);
                if ld <= rd { Some(l) } else { Some(r) }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    pub fn flatten(&self) -> (Vec<u32>, Vec<GpuBvhNode>) {
        let mut nodes: Vec<GpuBvhNode> = Vec::new();
        let mut object_ids: Vec<u32> = Vec::new();
        
        self.flatten_recursive(&mut nodes, &mut object_ids);

        return (object_ids, nodes)
    }

    pub fn flatten_recursive(&self, nodes: &mut Vec<GpuBvhNode>, object_ids: &mut Vec<u32>) -> (Vec<u32>, Vec<GpuBvhNode>) {
        todo!()
    }

}



pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        BoundingBox { min, max }
    }

    pub fn new_empty() -> Self {
        BoundingBox {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn grow_to_fit(&mut self, new_shape_id: u32) {
        if let Some(shape) = get_GLOBAL().get_object_by_id(new_shape_id) {
            let shape_min = shape.get_min_bounds();
            let shape_max = shape.get_max_bounds();
            if self.min.x < shape_min.x {
                self.min.x = shape_min.x;
            }
            if self.min.y < shape_min.y {
                self.min.y = shape_min.y;
            }
            if self.min.z < shape_min.z {
                self.min.z = shape_min.z;
            }
            if self.max.x > shape_max.x {
                self.max.x = shape_max.x;
            }
            if self.max.y > shape_max.y {
                self.max.y = shape_max.y;
            }
            if self.max.z > shape_max.z {
                self.max.z = shape_max.z;
            }
        }
    }


}