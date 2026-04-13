use crate::{BVH::BoundingBox, intersection::{Hit, Intersection}, ray::Ray, triangle::Triangle, utils::{Transform, get_GLOBAL}, vector::Vec3};



pub struct TriangleBVH {
    pub ID: u32,
    pub nodes: Vec<TBVHNodeType>,
    pub bounding_box: BoundingBox,
    pub transform: Transform,
}

impl TriangleBVH {
    pub fn new(triangles: &[Triangle]) -> Self {
        Self::build(triangles, Vec3::new(0.0, 0.0, 0.0), 1.0)
    }

    pub fn new_with_transform(triangles: &[Triangle], pos: Vec3, scale: f32) -> Self {
        Self::build(triangles, pos, scale)
    }


    pub fn build(triangles: &[Triangle], position: Vec3, scale: f32) -> Self {
        let mut root = TriangleBVH { 
            ID: get_GLOBAL().next_object_id(), 
            nodes: Vec::new(), 
            bounding_box: BoundingBox::new_empty(), 
            transform: Transform::new(position, scale),
        };

        for tri in triangles {
            root.bounding_box.grow_to_fit_triangle(tri);
        }

        root.fill_nodes_recursive(triangles.to_vec(), 0);

        root

    }

    pub fn fill_nodes_recursive(&mut self, triangles: Vec<Triangle>, depth: usize) -> u32 {
        let mut new_bounding_box = BoundingBox::new_empty();

        for tri in &triangles {
            new_bounding_box.grow_to_fit_triangle(tri);
        }

        if depth >= 20 || triangles.len() <= 2 {

            self.nodes.push(TBVHNodeType::leaf(depth as u32, triangles, new_bounding_box));
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

        let mut sorted_tris = triangles;
        sorted_tris.sort_by(|a, b| {
            let a_centroid = (a.p1 + a.p2 + a.p3) / 3.0;
            let b_centroid = (b.p1 + b.p2 + b.p3) / 3.0;
            let a_proj = a_centroid.dot(principal_axis);
            let b_proj = b_centroid.dot(principal_axis);
            a_proj.partial_cmp(&b_proj).unwrap()
        });

        let mid = sorted_tris.len() / 2;
        let left_tris = sorted_tris[..mid].to_vec();
        let right_tris = sorted_tris[mid..].to_vec();

        let right_child_id = self.fill_nodes_recursive(right_tris, depth + 1);
        let left_child_id = self.fill_nodes_recursive(left_tris, depth + 1);

        self.nodes.push(TBVHNodeType::node(depth as u32, right_child_id, new_bounding_box));
        (self.nodes.len() - 1) as u32

    }

    pub fn traverse(&self, ray: &Ray) -> Option<Intersection> {
        if self.world_bounding_box().hit(ray).is_none() {
            return None;
        }

        let local_ray = Ray::new(
            (ray.origin - self.transform.position) / self.transform.scale,
            ray.direction / self.transform.scale,
            ray.color,
        );

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

            let node_tmin = match node_bb.hit(&local_ray) {
                Some(t) => t,
                None => continue,
            };

            if node_tmin >= best_distance {
                continue;
            }

            match node {
                TBVHNodeType::TBVHLeaf(leaf) => {
                    for tri in &leaf.tris {
                        if let Some(intersection) = tri.intersect(&local_ray) {
                            if let Some(hit) = intersection.hitdata.as_ref() {
                                if hit.distance > 0.001 && hit.distance < best_distance {
                                    best_distance = hit.distance;
                                    best_hit = Some(intersection);
                                }
                            }
                        }
                    }
                }
                TBVHNodeType::TBVHNode(internal) => {
                    let right_index = internal.right_child as usize;
                    let left_index = node_index - 1;

                    let left_t = self.nodes[left_index].bounding_box().hit(&local_ray);
                    let right_t = self.nodes[right_index].bounding_box().hit(&local_ray);

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

    

    //////////////////////////////////////////////////////////////////////////////
    // utils
    //////////////////////////////////////////////////////////////////////////////
    
    pub fn set_position(&mut self, position: Vec3) {
        self.transform.position = position;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.transform.scale = scale;
    }

    pub fn world_bounding_box(&self) -> BoundingBox {
        let t = &self.transform;
        BoundingBox {
            min: t.transform_point(self.bounding_box.min),
            max: t.transform_point(self.bounding_box.max),
        }
    }

}


pub struct TBVHNode {
    internal_ID: u32,
    pub right_child: u32,
    pub bounding_box: BoundingBox,
}

impl TBVHNode {
    pub fn new(id: u32, right_child: u32, bb: BoundingBox) -> Self {
        TBVHNode {internal_ID: id, right_child, bounding_box: bb}
    }
}

pub struct TBVHLeaf {
    pub internal_ID: u32,
    pub tris: Vec<Triangle>,
    pub bounding_box: BoundingBox,
}

impl TBVHLeaf {
    pub fn new(id: u32, tris: Vec<Triangle>, bb: BoundingBox) -> Self {
        TBVHLeaf { internal_ID: id, tris, bounding_box: bb }
    }
}

pub enum TBVHNodeType {
    TBVHLeaf(TBVHLeaf),
    TBVHNode(TBVHNode),
}

impl TBVHNodeType {
    pub fn leaf(id: u32, tris: Vec<Triangle>, bb: BoundingBox) -> Self {
        TBVHNodeType::TBVHLeaf(TBVHLeaf::new(id, tris, bb))
    }

    pub fn node(id: u32, right_child: u32, bb: BoundingBox) -> Self {
        TBVHNodeType::TBVHNode(TBVHNode::new(id, right_child, bb))
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        match self {
            TBVHNodeType::TBVHLeaf(leaf) => &leaf.bounding_box,
            TBVHNodeType::TBVHNode(node) => &node.bounding_box,
        }
    }

    pub fn id (&self) -> u32 {
        match self {
            TBVHNodeType::TBVHLeaf(leaf) => leaf.internal_ID,
            TBVHNodeType::TBVHNode(_) => 0, // Internal nodes don't have a scene object ID
        }
    }
}