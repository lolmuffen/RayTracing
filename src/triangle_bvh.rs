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

        root.fill_nodes_recursive(triangles.to_vec(), 0);

        root

    }

    pub fn fill_nodes_recursive(&mut self, triangles: Vec<Triangle>, depth: usize) -> u32 {

        for tri in &triangles {
            self.bounding_box.grow_to_fit_triangle(tri);
        }

        if depth >= 20 || triangles.len() <= 2 {
            self.nodes.push(TBVHNodeType::leaf(depth as u32, triangles));
            return (self.nodes.len() - 1) as u32;
        }

        let centroid = (self.bounding_box.min + self.bounding_box.max) * 0.5;
        let size = self.bounding_box.max - self.bounding_box.min;
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

        let left_child_id = self.fill_nodes_recursive(left_tris, depth + 1);
        let right_child_id = self.fill_nodes_recursive(right_tris, depth + 1);

        self.nodes.push(TBVHNodeType::node(left_child_id, right_child_id));
        (self.nodes.len() - 1) as u32

    }

    pub fn traverse(&self, ray: &Ray) -> Option<Intersection> {
        if self.world_bounding_box().hit(ray).is_none() {
            return None;
        }

        let local_ray = Ray::new(
            (ray.origin - self.transform.position) / self.transform.scale,
            ray.direction / self.transform.scale,
            ray.color
        );

        let mut closest: Option<Intersection> = None;

        let mut stack = vec![0]; // Start with root node index

        while let Some(node_index) = stack.pop() {
            match &self.nodes[node_index as usize] {
                TBVHNodeType::TBVHNode(node) => {
                    // Internal node: push children onto stack
                    stack.push(node.right_child);
                    stack.push(node.left_child); // Left child
                }
                TBVHNodeType::TBVHLeaf(leaf) => {
                    // Leaf node: check all triangles for intersection
                    for tri in &leaf.tris {
                        if let Some(intersection) = tri.intersect(&local_ray) {
                            if let Some(hit) = &intersection.hitdata {
                                let world_t = hit.distance * self.transform.scale;
                                if world_t <= 0.001 { continue; } // skip if too close or behind
                                let world_hit_point = hit.hit_point * self.transform.scale + self.transform.position;
                                let world_normal = hit.normal / self.transform.scale;
                                let adjusted_hit = Hit::new_with_material(world_t, world_hit_point, world_normal, hit.front_face, hit.material.clone().unwrap());
                                let adjusted_intersection = Intersection::new(true, Some(adjusted_hit), Some(self.ID));
                                if closest.is_none() || world_t < closest.as_ref().unwrap().hitdata.as_ref().unwrap().distance {
                                    closest = Some(adjusted_intersection);
                                }
                            }
                        }
                    }
                }
            }
        }
        closest
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
    pub left_child: u32,
    pub right_child: u32,
}

impl TBVHNode {
    pub fn new(left_child: u32, right_child: u32) -> Self {
        TBVHNode {left_child, right_child}
    }
}

pub struct TBVHLeaf {
    pub internal_ID: u32,
    pub tris: Vec<Triangle>,
}

impl TBVHLeaf {
    pub fn new(id: u32, tris: Vec<Triangle>) -> Self {
        TBVHLeaf { internal_ID: id, tris }
    }
}

pub enum TBVHNodeType {
    TBVHLeaf(TBVHLeaf),
    TBVHNode(TBVHNode),
}

impl TBVHNodeType {
    pub fn leaf(id: u32, tris: Vec<Triangle>) -> Self {
        TBVHNodeType::TBVHLeaf(TBVHLeaf::new(id, tris))
    }

    pub fn node(left_child: u32, right_child: u32) -> Self {
        TBVHNodeType::TBVHNode(TBVHNode::new(left_child, right_child))
    }
}