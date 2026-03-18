use crate::ray::Ray;
use crate::triangle_bvh::TriangleBVH;
use crate::utils::get_GLOBAL;
use crate::triangle::Triangle;
use crate::vector::Vec3;
use crate::intersection::{Hit, Intersection};
use crate::material::Material;
use crate::sphere::Sphere;


pub enum Shape {
    Sphere {sphere: Sphere},
    Triangle{tri: Triangle},
    TriangleMesh{tri_mesh: TriangleBVH}
}

impl Shape {
    pub fn sphere(position: Vec3, radius: f32, material: Material) -> Shape {
        let sphere = Sphere {
            ID: get_GLOBAL().next_object_id(), 
            position, 
            radius, 
            material 
        };
        Shape::Sphere { sphere }
    }

    pub fn triangle(p1: Vec3, p2: Vec3, p3: Vec3, mat: Material) -> Shape {
        let e1 = p2 - p1;
        let e2 = p3 - p1;
        let normal = e1.cross(e2).normalize();
        Shape::Triangle { tri: Triangle { p1, p2, p3, normal, material: mat, id: get_GLOBAL().next_object_id() } }
    }

    pub fn triangle_mesh(tris: &[Triangle], position: Vec3, scale: f32) -> Shape {
        Shape::TriangleMesh { tri_mesh: TriangleBVH::new_transformed(tris, position, scale)}
    }


    pub fn intersect(&self, ray: &Ray) -> Intersection {
        match self {
            Shape::Sphere { sphere } => {let oc = ray.origin - sphere.position;
                                let a = ray.direction.dot(ray.direction);
                                let b = 2.0 * oc.dot(ray.direction);
                                let c = oc.dot(oc) - sphere.radius * sphere.radius;
                                let discriminant = b * b - 4.0 * a * c;

                                if discriminant < 0.0 {
                                    Intersection::new(false, None, None)
                                } else {
                                    let sqrt_disc = discriminant.sqrt();
                                    let t1 = (-b - sqrt_disc) / (2.0 * a);
                                    let t2 = (-b + sqrt_disc) / (2.0 * a);
                                    let t = if t1 > 0.001 { t1 } else { t2 };


                                    if t > 0.001 {
                                        let hit_point = ray.origin + ray.direction * t;

                                        let normal = (hit_point - sphere.position).normalize();
                                        let hit = Hit::new(t, hit_point, normal);

                                        Intersection::new(true, Some(hit), Some(sphere.ID))
                                    } else {
                                        Intersection::new(false, None, None)
                                    }
                                }}
            Shape::Triangle { tri } => {
                                                //This function uses moller_trumbore_intersection
                                                //I dont fully understand this math

                                                let e1 = tri.p2 - tri.p1;
                                                let e2 = tri.p3 - tri.p1;

                                                let ray_cross_e2 = ray.direction.cross(e2);
                                                let det = e1.dot(ray_cross_e2);

                                                if det > -f32::EPSILON && det < f32::EPSILON {
                                                    return Intersection { hit: false, hitdata: None, object_id: None }; // This ray is parallel to this triangle.
                                                }

                                                let inv_det = 1.0 / det;
                                                let s = ray.origin - tri.p1;
                                                let u = inv_det * s.dot(ray_cross_e2);
                                                if u < 0.0 || u > 1.0 {
                                                    return Intersection { hit: false, hitdata: None, object_id: None };
                                                }

                                                let s_cross_e1 = s.cross(e1);
                                                let v = inv_det * ray.direction.dot(s_cross_e1);
                                                if v < 0.0 || u + v > 1.0 {
                                                    return Intersection { hit: false, hitdata: None, object_id: None };
                                                }
                                                // At this stage we can compute t to find out where the intersection point is on the line.
                                                let t = inv_det * e2.dot(s_cross_e1);

                                                if t > f32::EPSILON { // ray intersection
                                                    let intersection_point = ray.origin + ray.direction * t;
                                                    return Intersection { hit: true, hitdata: Some(Hit::new(t, intersection_point, tri.normal)), object_id: Some(tri.id) };
                                                }
                                                else { // This means that there is a line intersection but not a ray intersection.
                                                    return Intersection { hit: false, hitdata: None, object_id: None };
                                                }
            }

            Shape::TriangleMesh { tri_mesh } => {
                                                                match tri_mesh.traverse(ray) {
                                                                    Some(intersection) => intersection,
                                                                    None => Intersection { hit: false, hitdata: None, object_id: None },
                                                                }
                                                            }  
        }
    }


    pub fn get_id(&self) -> u32 {
        match self {
            Self::Sphere { sphere } => {sphere.ID},
            Self::Triangle { tri } => {tri.id},
            Self::TriangleMesh { tri_mesh } => {tri_mesh.ID}
        }
    }

    pub fn get_min_bounds(&self) -> Vec3 {
        match self {
            Self::Sphere { sphere } => {
            Vec3::new(
                sphere.position.x - sphere.radius,
                sphere.position.y - sphere.radius,
                sphere.position.z - sphere.radius,
                )
        },
            Self::Triangle { tri } => {let min_x = tri.p1.x.min(tri.p2.x).min(tri.p3.x);
                                                 let min_y = tri.p1.y.min(tri.p2.y).min(tri.p3.y);
                                                 let min_z = tri.p1.z.min(tri.p2.z).min(tri.p3.z);
                                                 Vec3::new(min_x, min_y, min_z)
        },
            
            Self::TriangleMesh { tri_mesh } => tri_mesh.world_bounding_box().min,


        }
        

    }

    pub fn get_max_bounds(&self) -> Vec3 {
        match self {
            Self::Sphere { sphere } => {Vec3::new(
                                                    sphere.position.x + sphere.radius,
                                                    sphere.position.y + sphere.radius,
                                                    sphere.position.z + sphere.radius,
                                                )
        },
            Self::Triangle { tri } => {let max_x = tri.p1.x.max(tri.p2.x).max(tri.p3.x);
                                                let max_y = tri.p1.y.max(tri.p2.y).max(tri.p3.y);
                                                let max_z = tri.p1.z.max(tri.p2.z).max(tri.p3.z);
                                                return Vec3::new(max_x, max_y, max_z)
        },
            Self::TriangleMesh { tri_mesh } => tri_mesh.world_bounding_box().max,
        }


    }

    pub fn get_material(&self) -> Material {
        match self {
            Self::Sphere { sphere } => {sphere.material},
            Self::Triangle { tri } => {tri.material},
            Self::TriangleMesh { tri_mesh } => {panic!("get_material() called on TriangleMesh — use hit.material instead")}
        }
    }

}