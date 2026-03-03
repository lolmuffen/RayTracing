/// gpu_structs.rs
///
/// Every type here is written directly into a wgpu buffer and read by the
/// compute shader. Rules:
///   - #[repr(C)] — no Rust reordering
///   - bytemuck::Pod + Zeroable — safe zero-copy cast to &[u8]
///   - Every vec3 field is followed by a padding f32 so it aligns to 16 bytes
///     (WGSL's vec3<f32> is 12 bytes but has 16-byte alignment in uniform/storage)
///
/// Material kinds (matches WGSL constants in path_tracer.wgsl):
///   0 = Lambertian
///   1 = Metal
///   2 = Glass
///   3 = Volume
///   4 = Emissive
///   5 = Specular



#[repr(C)]
struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    material_id: u32,
    _pad: [u32; 3],
}

#[repr(C)]  
struct GpuTriangle {
    p1: [f32; 3], _p1: f32,
    p2: [f32; 3], _p2: f32,
    p3: [f32; 3], _p3: f32,
    normal: [f32; 3],
    material_id: u32,
}

// CPU-side, written once to a GPU buffer
#[repr(C)]
struct GpuBvhNode {
    bounding_box_min: [f32; 3],
    left_child: u32,  // if leaf: index into shape_ids array
    right_child: u32,
    bounding_box_max: [f32; 3],
    shape_count: u32,          // 0 = internal node, >0 = leaf
    first_shape: u32,
    // if internal: right child is always left_or_first_shape + 1 (or store explicitly)
    
    _pad: [u32; 3],
}


pub struct GpuMaterial {
    pub color:                [f32; 3],
    pub kind:                 u32,

    pub albedo:               f32,
    pub roughness:            f32,
    pub refraction_index:     f32,
    pub specular_probability: f32,

    pub intensity:            f32,
    pub _pad:                 [f32; 3],
}

// In BVH leaf nodes, shape IDs encode both type and index:
// high 4 bits = type (0=sphere, 1=triangle), low 28 bits = index