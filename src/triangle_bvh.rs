use crate::BVH::BoundingBox;
use crate::intersection::{Hit, Intersection};
use crate::ray::Ray;
use crate::triangle::Triangle;
use crate::vector::Vec3;
use crate::utils::{moller_trumbore, get_GLOBAL};

// =============================================================================
// Transform
// =============================================================================

/// Affine transform stored on the mesh root. The BVH itself is built in the
/// mesh's *local* space; at ray-test time we transform the ray into local space,
/// run the normal traversal, then transform the resulting hit back to world space.
///
/// This means the BVH never needs to be rebuilt when you move or scale a mesh —
/// only the two cheap transform calls per ray change.
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    /// World-space position of the mesh origin.
    pub position: Vec3,
    /// Uniform scale factor (applied before translation).
    pub scale: f32,
}

impl Transform {
    pub fn new(position: Vec3, scale: f32) -> Self {
        Self { position, scale }
    }

    /// Identity transform: origin at (0,0,0), scale 1.
    pub fn identity() -> Self {
        Self { position: Vec3::new(0.0, 0.0, 0.0), scale: 1.0 }
    }

    // -------------------------------------------------------------------------
    // Forward transforms  (local → world)
    // -------------------------------------------------------------------------

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        p * self.scale + self.position
    }

    pub fn transform_normal(&self, n: Vec3) -> Vec3 {
        // For a uniform scale the normal transform is the same as the point
        // transform (no translation, divide by scale² cancels to 1/scale which
        // re-normalises anyway).  We just need to renormalise after.
        (n / self.scale).normalize()
    }

    // -------------------------------------------------------------------------
    // Inverse transforms  (world → local)
    // -------------------------------------------------------------------------

    pub fn inverse_transform_point(&self, p: Vec3) -> Vec3 {
        (p - self.position) / self.scale
    }

    pub fn inverse_transform_direction(&self, d: Vec3) -> Vec3 {
        // Directions are not translated; only scale applies.
        d / self.scale
    }
}

// =============================================================================
// TriangleBVH
// =============================================================================

pub struct TriangleBVH {
    pub ID: u32,
    pub bounding_box: BoundingBox,   // always in *local* space
    pub left_child: Option<Box<TriangleBVH>>,
    pub right_child: Option<Box<TriangleBVH>>,
    pub tris: Option<Vec<Triangle>>, // leaf node owns its triangles
    /// Only meaningful on the root node; children ignore it.
    pub transform: Transform,
}

impl TriangleBVH {
    /// Build a BVH over `triangles` with an identity transform (position = origin, scale = 1).
    pub fn new(triangles: &[Triangle]) -> Self {
        Self::new_transformed(triangles, Vec3::new(0.0, 0.0, 0.0), 1.0)
    }

    /// Build a BVH over `triangles` and attach a world-space position and uniform scale.
    ///
    /// A **single** global object ID is claimed here for the root node.  Internal
    /// child nodes and leaf triangles are not scene objects and use ID 0.
    pub fn new_transformed(triangles: &[Triangle], position: Vec3, scale: f32) -> Self {
        // Claim the one real scene-object ID *before* building the tree.
        // Triangle::new / new_with_normal no longer call next_object_id(), so
        // nothing inside build_recursive will consume IDs.
        let root_id = get_GLOBAL().next_object_id();
        let indices: Vec<usize> = (0..triangles.len()).collect();
        let mut root = TriangleBVH::build_recursive(triangles, &indices, 0, 0);
        root.ID = root_id;
        root.transform = Transform::new(position, scale);
        root
    }

    /// Update the world-space position of the mesh without rebuilding the BVH.
    pub fn set_position(&mut self, position: Vec3) {
        self.transform.position = position;
    }

    /// Update the uniform scale of the mesh without rebuilding the BVH.
    pub fn set_scale(&mut self, scale: f32) {
        self.transform.scale = scale;
    }

    /// The world-space AABB of this mesh, derived from the local bounding box
    /// and the current transform.  Used by the scene BVH.
    pub fn world_bounding_box(&self) -> BoundingBox {
        let t = &self.transform;
        BoundingBox {
            min: t.transform_point(self.bounding_box.min),
            max: t.transform_point(self.bounding_box.max),
        }
    }

    // -------------------------------------------------------------------------
    // Internal recursive builder  (works entirely in local space)
    // -------------------------------------------------------------------------

    fn build_recursive(triangles: &[Triangle], indices: &[usize], depth: usize, child_id: u32) -> Self {
        let mut bounding_box = BoundingBox::new_empty();

        for &i in indices {
            bounding_box.grow_to_fit_triangle(&triangles[i]);
        }

        if depth >= 20 || indices.len() <= 2 {
            let leaf_tris = indices.iter().map(|&i| triangles[i].clone()).collect();
            return TriangleBVH {
                bounding_box,
                left_child: None,
                right_child: None,
                tris: Some(leaf_tris),
                ID: child_id,
                transform: Transform::identity(),
            };
        }

        let centroid = (bounding_box.min + bounding_box.max) * 0.5;

        let size = bounding_box.max - bounding_box.min;
        let principal_axis = if size.x >= size.y && size.x >= size.z {
            Vec3::new(1.0, 0.0, 0.0)
        } else if size.y >= size.z {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };

        let mut sorted_indices = indices.to_vec();
        sorted_indices.sort_by(|&a, &b| {
            let ca = (triangles[a].p1 + triangles[a].p2 + triangles[a].p3) * (1.0 / 3.0);
            let cb = (triangles[b].p1 + triangles[b].p2 + triangles[b].p3) * (1.0 / 3.0);
            let proj_a = (ca - centroid).dot(principal_axis);
            let proj_b = (cb - centroid).dot(principal_axis);
            proj_a.partial_cmp(&proj_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = sorted_indices.len() / 2;
        let left_child  = Box::new(TriangleBVH::build_recursive(triangles, &sorted_indices[..mid],  depth + 1, 1));
        let right_child = Box::new(TriangleBVH::build_recursive(triangles, &sorted_indices[mid..], depth + 1, 2));

        TriangleBVH {
            bounding_box,
            left_child: Some(left_child),
            right_child: Some(right_child),
            tris: None,
            ID: child_id,
            transform: Transform::identity(),
        }
    }

    // -------------------------------------------------------------------------
    // Public traversal entry point  (root call only)
    // -------------------------------------------------------------------------

    /// Intersect a world-space ray against the mesh.
    ///
    /// The ray is transformed into local space, the internal BVH is traversed
    /// in local space, and the resulting hit point + normal are transformed back
    /// to world space before returning.
    pub fn traverse(&self, ray: &Ray) -> Option<Intersection> {
        let t = &self.transform;
        let local_origin    = t.inverse_transform_point(ray.origin);
        let local_direction = t.inverse_transform_direction(ray.direction);
        let local_ray = Ray::new(local_origin, local_direction, ray.color);

        let local_hit = self.traverse_local(&local_ray, self.ID)?;  // pass root ID here
        Some(transform_intersection(local_hit, t))
    }

    /// Internal recursive traversal; works entirely in the local coordinate frame.
   fn traverse_local(&self, ray: &Ray, root_id: u32) -> Option<Intersection> {
        if !self.bounding_box.hit(ray) {
            return None;
        }

        if let Some(tris) = &self.tris {
            let mut best_hit: Option<Hit> = None;

            for tri in tris {
                if let Some((t, hit_point)) = moller_trumbore(ray, tri) {
                    if t > 0.001 {
                        let is_better = match &best_hit {
                            Some(bh) => t < bh.distance,
                            None => true,
                        };
                        if is_better {
                            let front_face = ray.direction.dot(tri.normal) < 0.0;
                            let normal = if front_face { tri.normal } else { -tri.normal };
                            best_hit = Some(Hit::new_with_material(t, hit_point, normal, front_face, tri.material));
                        }
                    }
                }
            }

            return best_hit.map(|h| Intersection::new(true, Some(h), Some(root_id)));
        }

        let left_hit  = self.left_child.as_ref().and_then(|c| c.traverse_local(ray, root_id));
        let right_hit = self.right_child.as_ref().and_then(|c| c.traverse_local(ray, root_id));

        match (left_hit, right_hit) {
            (Some(l), Some(r)) => {
                let ld = l.hitdata.as_ref().map(|h| h.distance).unwrap_or(f32::INFINITY);
                let rd = r.hitdata.as_ref().map(|h| h.distance).unwrap_or(f32::INFINITY);
                if ld <= rd { Some(l) } else { Some(r) }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}

// =============================================================================
// Helper: transform a local-space Intersection back to world space
// =============================================================================

fn transform_intersection(mut inter: Intersection, t: &Transform) -> Intersection {
    if let Some(ref mut hit) = inter.hitdata {
        // The hit point stored by moller_trumbore is in local space; bring it back.
        hit.hit_point = t.transform_point(hit.hit_point);
        // Normals transform inversely to positions (transpose of inverse).
        // For a uniform scale this simply renormalises after dividing by scale.
        hit.normal = t.transform_normal(hit.normal);

        // The parametric distance `t` returned by Möller–Trumbore was computed
        // against the *scaled* local ray direction, so it needs to be corrected
        // to match world-space distance:  t_world = t_local * scale.
        hit.distance *= t.scale;
    }
    inter
}