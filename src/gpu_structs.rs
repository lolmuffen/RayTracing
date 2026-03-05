// gpu_structs.rs  (updated)
//
// All GPU-facing types derive bytemuck::Pod + Zeroable so they can be cast
// directly to &[u8] for wgpu buffer uploads.
//
// Alignment rules (matches WGSL storage buffer layout):
//   - Every vec3<f32> field is padded to 16 bytes (add a trailing f32 / u32)
//   - Every struct is a multiple of 16 bytes total

// ---------------------------------------------------------------------------
// Material  (kind constants match path_tracer.wgsl)
// ---------------------------------------------------------------------------
//  0 = Lambertian
//  1 = Metal
//  2 = Glass
//  3 = Volume
//  4 = Emissive
//  5 = Specular

#[repr(C)]
pub struct GpuMaterial {
    pub color:                [f32; 3],
    pub kind:                 u32,        // See constants above

    pub albedo:               f32,
    pub roughness:            f32,
    pub refraction_index:     f32,
    pub specular_probability: f32,

    pub intensity:            f32,
    pub _pad:                 [f32; 3],   // pad to 48 bytes (3 × 16)
}

// ---------------------------------------------------------------------------
// Sphere
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct GpuSphere {
    pub center:      [f32; 3],
    pub radius:      f32,
    pub material_id: u32,
    pub _pad:        [u32; 3],            // pad to 32 bytes (2 × 16)
}

// ---------------------------------------------------------------------------
// Triangle
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct GpuTriangle {
    pub p1: [f32; 3], pub _p1: f32,      // 16 bytes
    pub p2: [f32; 3], pub _p2: f32,      // 16 bytes
    pub p3: [f32; 3], pub _p3: f32,      // 16 bytes
    pub normal:      [f32; 3],
    pub material_id: u32,                 // 16 bytes  — total 64 bytes
}

// ---------------------------------------------------------------------------
// BVH node
//
// Layout in WGSL:
//   bb_min      vec3<f32>  + left_child  u32   = 16 bytes
//   right_child u32        + bb_max      vec3<f32>  needs care:
//     WGSL vec3 has 16-byte alignment, so we must match that here.
//   We use the layout below which keeps everything at 16-byte boundaries.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct GpuBvhNode {
    pub bounding_box_min: [f32; 3],
    pub left_child:       u32,            // 16 bytes

    pub right_child:      u32,
    pub bounding_box_max: [f32; 3],       // NOTE: right_child is the first u32,
                                          // then the vec3 — matches WGSL field order

    pub shape_count:      u32,            // 0 = internal node, >0 = leaf
    pub first_shape:      u32,            // index into bvh_shape_ids array
    pub _pad:             [u32; 2],       // pad to 48 bytes total (3 × 16)
}