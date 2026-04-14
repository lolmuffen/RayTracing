use crate::intersection::Intersection;
use crate::ray::Ray;
use crate::shape::Shape; 
use crate::utils::{BoundingBox, Global, get_GLOBAL};
use crate::vector::Vec3;


pub struct sceneBVH<'a> {
    pub ID: u32, 
    pub bounding_box: BoundingBox,
    pub nodes: Vec<sceneBVHNodeType<'a>>,
}

impl<'a> sceneBVH<'a> {
    pub fn new() -> Self {
        sceneBVH::build()
    }

    fn build() -> Self {
        let mut root = sceneBVH { 
            ID: get_GLOBAL().next_object_id(), 
            nodes: Vec::new(), 
            bounding_box: BoundingBox::new_empty(),
        };

        let shape_ids: Vec<u32> = get_GLOBAL()
        .get_objects()
        .unwrap()  
        .iter()
        .map(|obj| obj.get_id())
        .collect();

        for object_id in &shape_ids {
            root.bounding_box.grow_to_fit(*object_id);
        }

        root.build_recursive(shape_ids, 0);

        root

    }

    pub fn build_recursive(&mut self, shape_ids: Vec<u32>, depth: usize) -> u32 {
        let mut new_bounding_box = BoundingBox::new_empty();

        for shape_id in &shape_ids {
            new_bounding_box.grow_to_fit(*shape_id);
        }

        if shape_ids.len() == 1 {
            let shape = get_GLOBAL().get_object_by_id(*shape_ids.get(0).unwrap()).unwrap();
            
            self.nodes.push(sceneBVHNodeType::leaf(depth as u32, shape, new_bounding_box));
            return (self.nodes.len() - 1) as u32;
        }

        let centroid = (new_bounding_box.min + new_bounding_box.max) * 0.5;
        let size = new_bounding_box.max - new_bounding_box.min;
        let principal_axis = if size.x >= size.y && size.x >= size.z {
            Vec3::new(1.0, 0.0, 0.0)
        } else if size.y >= size.z {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };

        let mut sorted_shapes = shape_ids;
        sorted_shapes.sort_by(|&a, &b| {
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

            let projection_a = center_a.x * principal_axis.x + center_a.y * principal_axis.y + center_a.z * principal_axis.z;
            let projection_b = center_b.x * principal_axis.x + center_b.y * principal_axis.y + center_b.z * principal_axis.z;

            projection_a.partial_cmp(&projection_b).unwrap()
        });

        let mid = sorted_shapes.len() / 2;
        let left_ids = sorted_shapes[..mid].to_vec();
        let right_ids = sorted_shapes[mid..].to_vec();

        let right_child_id = self.build_recursive(right_ids, depth + 1);
        let left_child_id = self.build_recursive(left_ids, depth + 1);

        self.nodes.push(sceneBVHNodeType::node(depth as u32, right_child_id, new_bounding_box));
        (self.nodes.len() - 1) as u32

    }

    pub fn traverse(&self, ray: &Ray, global: &Global) -> Option<Intersection> {
        if self.bounding_box.hit(ray).is_none() {
            return None;
        }

        if self.nodes.is_empty() {
            return None;
        }

        let mut stack = Vec::with_capacity(64);
        let mut best_hit: Option<Intersection> = None;
        let mut best_distance = f32::INFINITY;

        stack.push(self.nodes.len() - 1);

        while let Some(node_index) = stack.pop() {
            let node = &self.nodes[node_index];
            let node_bb = node.bounding_box();

            let node_tmin = match node_bb.hit(&ray) {
                Some(t) => t,
                None => continue,
            };

            if node_tmin >= best_distance {
                continue;
            }

            match node {
                sceneBVHNodeType::sceneBVHLeaf(leaf) => {
                    
                    let intersection = leaf.shape.intersect(&ray);
                    if let Some(hit) = intersection.hitdata.as_ref() {
                        if hit.distance > 0.001 && hit.distance < best_distance {
                            best_distance = hit.distance;
                            best_hit = Some(intersection);
                        }
                    }
                    
                
                }
                sceneBVHNodeType::sceneBVHNode(internal) => {
                    let right_index = internal.right_child as usize;
                    let left_index = node_index - 1;

                    let left_t = self.nodes[left_index].bounding_box().hit(&ray);
                    let right_t = self.nodes[right_index].bounding_box().hit(&ray);

                    let push_child = |stack: &mut Vec<usize>, idx: usize, tmin: f32, best_distance: f32| {
                        if tmin < best_distance {
                            stack.push(idx);
                        }
                    };

                    match (left_t, right_t) {
                        (Some(lt), Some(rt)) => {
                            if lt <= rt {
                                push_child(&mut stack, right_index, rt, best_distance);
                                push_child(&mut stack, left_index, lt, best_distance);
                            } else {
                                push_child(&mut stack, left_index, lt, best_distance);
                                push_child(&mut stack, right_index, rt, best_distance);
                            }
                        }
                        (Some(lt), None) => push_child(&mut stack, left_index, lt, best_distance),
                        (None, Some(rt)) => push_child(&mut stack, right_index, rt, best_distance),
                        (None, None) => {}
                    }
                }
            }
        }

        best_hit
    }
}




pub struct sceneBVHLeaf<'a> {
    pub internal_ID: u32,
    pub shape: &'a Shape,
    pub bounding_box: BoundingBox,
}

pub struct sceneBVHNode {
    pub internal_ID: u32,
    pub right_child: u32,
    pub bounding_box: BoundingBox,
}


pub enum sceneBVHNodeType<'a> {
    sceneBVHLeaf(sceneBVHLeaf<'a>),
    sceneBVHNode(sceneBVHNode),
}

impl<'a> sceneBVHNodeType<'a> {
    pub fn leaf(id: u32, shape: &'a Shape, bb: BoundingBox) -> Self {
        sceneBVHNodeType::sceneBVHLeaf(sceneBVHLeaf { internal_ID: id, shape, bounding_box: bb})
    }

    pub fn node(id: u32, right_child: u32, bb: BoundingBox) -> Self {
        sceneBVHNodeType::sceneBVHNode(sceneBVHNode { internal_ID: id, right_child, bounding_box: bb})
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        match self {
            sceneBVHNodeType::sceneBVHLeaf(leaf) => &leaf.bounding_box,
            sceneBVHNodeType::sceneBVHNode(node) => &node.bounding_box,
        }
    }

    pub fn id (&self) -> u32 {
        match self {
            sceneBVHNodeType::sceneBVHLeaf(leaf) => leaf.internal_ID,
            sceneBVHNodeType::sceneBVHNode(_) => 0, // Internal nodes don't have a scene object ID
        }
    }
}
