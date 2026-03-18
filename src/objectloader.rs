use std::fs;
use std::path::Path;
use crate::material::Material;
use crate::triangle::Triangle;
use crate::vector::Vec3;

// =============================================================================
// Error type
// =============================================================================

#[derive(Debug)]
pub enum ObjLoadError {
    IoError(std::io::Error),
    ParseError { line_number: usize, message: String },
    FaceReferencesOutOfBounds { line_number: usize, index: usize, max: usize },
}

impl std::fmt::Display for ObjLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjLoadError::IoError(e) => write!(f, "IO error: {}", e),
            ObjLoadError::ParseError { line_number, message } => {
                write!(f, "Parse error on line {}: {}", line_number, message)
            }
            ObjLoadError::FaceReferencesOutOfBounds { line_number, index, max } => {
                write!(
                    f,
                    "Face on line {} references vertex index {} but only {} vertices exist",
                    line_number, index, max
                )
            }
        }
    }
}

impl From<std::io::Error> for ObjLoadError {
    fn from(e: std::io::Error) -> Self {
        ObjLoadError::IoError(e)
    }
}

// =============================================================================
// Public loader
// =============================================================================

/// Load a Wavefront `.obj` file and return all faces as a `Vec<Triangle>`.
///
/// # Supported features
/// - `v`  — vertex positions
/// - `vn` — vertex normals (used when present; face normal is computed otherwise)
/// - `f`  — faces with any of the three OBJ face formats:
///            `f v`, `f v/vt`, `f v/vt/vn`, `f v//vn`
///   Faces with more than 3 vertices are fan-triangulated automatically.
/// - `#`  — comments (ignored)
/// - `o`, `g`, `s`, `usemtl`, `mtllib` lines are silently skipped
///
/// # Arguments
/// * `path`     — path to the `.obj` file
/// * `material` — material applied to every triangle in the mesh
///
/// # Example
/// ```rust
/// let tris = load_obj("assets/bunny.obj", Material::lambertian(1.0, Vec3::new(0.8, 0.8, 0.8)))?;
/// ```
pub fn load_obj<P: AsRef<Path>>(path: P, material: Material) -> Result<Vec<Triangle>, ObjLoadError> {
    let source = fs::read_to_string(path)?;
    parse_obj(&source, material)
}

// =============================================================================
// Parser
// =============================================================================

fn parse_obj(source: &str, material: Material) -> Result<Vec<Triangle>, ObjLoadError> {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals:   Vec<Vec3> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_number = line_idx + 1;

        // Strip inline comments and surrounding whitespace
        let line = match raw_line.find('#') {
            Some(i) => &raw_line[..i],
            None    => raw_line,
        }.trim();

        if line.is_empty() {
            continue;
        }

        // Split into keyword + rest
        let mut tokens = line.split_whitespace();
        let keyword = match tokens.next() {
            Some(k) => k,
            None    => continue,
        };

        match keyword {
            // ------------------------------------------------------------------
            // Vertex position:  v x y z [w]
            // ------------------------------------------------------------------
            "v" => {
                let pos = parse_vec3(tokens, line_number)?;
                positions.push(pos);
            }

            // ------------------------------------------------------------------
            // Vertex normal:  vn x y z
            // ------------------------------------------------------------------
            "vn" => {
                let n = parse_vec3(tokens, line_number)?;
                normals.push(n.normalize());
            }

            // ------------------------------------------------------------------
            // Texture coordinate:  vt u [v [w]]  — parsed but not stored
            // ------------------------------------------------------------------
            "vt" => { /* ignored */ }

            // ------------------------------------------------------------------
            // Face:  f v1[/vt1[/vn1]] v2[/vt2[/vn2]] ...
            // Supports triangles and polygons (fan-triangulated).
            // ------------------------------------------------------------------
            "f" => {
                let mut face_verts: Vec<FaceVertex> = Vec::new();
                for tok in tokens {
                    face_verts.push(parse_face_vertex(tok, line_number)?);
                }

                if face_verts.len() < 3 {
                    return Err(ObjLoadError::ParseError {
                        line_number,
                        message: format!("face has only {} vertices, need at least 3", face_verts.len()),
                    });
                }

                // Validate and resolve all indices before building triangles
                let mut resolved: Vec<ResolvedVertex> = Vec::new();
                for fv in &face_verts {
                    resolved.push(resolve_vertex(fv, &positions, &normals, line_number)?);
                }

                // Fan triangulation: (0,1,2), (0,2,3), (0,3,4), …
                for i in 1..resolved.len() - 1 {
                    let a = &resolved[0];
                    let b = &resolved[i];
                    let c = &resolved[i + 1];

                    let triangle = build_triangle(a, b, c, material);
                    triangles.push(triangle);
                }
            }

            // ------------------------------------------------------------------
            // Silently skip everything else (o, g, s, mtllib, usemtl, …)
            // ------------------------------------------------------------------
            _ => {}
        }
    }

    Ok(triangles)
}

// =============================================================================
// Internal helpers
// =============================================================================

/// A face vertex as it appears in the OBJ file (1-based, possibly negative).
struct FaceVertex {
    pos_index:    i32,
    normal_index: Option<i32>,
}

/// A face vertex after index resolution (0-based, validated).
struct ResolvedVertex {
    position: Vec3,
    normal:   Option<Vec3>,
}

/// Parse `v`, `v/vt`, `v/vt/vn`, or `v//vn` into a `FaceVertex`.
fn parse_face_vertex(token: &str, line_number: usize) -> Result<FaceVertex, ObjLoadError> {
    let parts: Vec<&str> = token.split('/').collect();

    let pos_index = parts[0].parse::<i32>().map_err(|_| ObjLoadError::ParseError {
        line_number,
        message: format!("invalid vertex index '{}'", parts[0]),
    })?;

    // Normal index is the third slash-separated field if present and non-empty
    let normal_index = if parts.len() >= 3 && !parts[2].is_empty() {
        Some(parts[2].parse::<i32>().map_err(|_| ObjLoadError::ParseError {
            line_number,
            message: format!("invalid normal index '{}'", parts[2]),
        })?)
    } else {
        None
    };

    Ok(FaceVertex { pos_index, normal_index })
}

/// Resolve OBJ-style 1-based (or negative) indices into 0-based Vec indices
/// and look up the actual data.
fn resolve_vertex(
    fv:          &FaceVertex,
    positions:   &[Vec3],
    normals:     &[Vec3],
    line_number: usize,
) -> Result<ResolvedVertex, ObjLoadError> {
    let pos_idx = resolve_index(fv.pos_index, positions.len(), line_number)?;
    let position = positions[pos_idx];

    let normal = if let Some(ni) = fv.normal_index {
        let idx = resolve_index(ni, normals.len(), line_number)?;
        Some(normals[idx])
    } else {
        None
    };

    Ok(ResolvedVertex { position, normal })
}

/// Convert a 1-based (positive or negative) OBJ index to a 0-based Rust index.
fn resolve_index(raw: i32, len: usize, line_number: usize) -> Result<usize, ObjLoadError> {
    let idx = if raw < 0 {
        // Negative indices count backwards from the end
        (len as i32 + raw) as usize
    } else {
        (raw - 1) as usize
    };

    if idx >= len {
        return Err(ObjLoadError::FaceReferencesOutOfBounds {
            line_number,
            index: raw as usize,
            max:   len,
        });
    }

    Ok(idx)
}

/// Build a `Triangle` from three resolved vertices.
/// Uses the per-vertex OBJ normals averaged across the face if available,
/// otherwise computes the geometric face normal from the positions.
fn build_triangle(a: &ResolvedVertex, b: &ResolvedVertex, c: &ResolvedVertex, material: Material) -> Triangle {
    let normal = match (a.normal, b.normal, c.normal) {
        (Some(na), Some(nb), Some(nc)) => {
            // Average the three vertex normals and re-normalise
            (na + nb + nc).normalize()
        }
        _ => {
            // Compute geometric normal from vertex positions
            let e1 = b.position - a.position;
            let e2 = c.position - a.position;
            e1.cross(e2).normalize()
        }
    };

    Triangle::new_with_normal(a.position, b.position, c.position, normal, material)
}

/// Parse three consecutive whitespace-separated floats into a `Vec3`.
fn parse_vec3<'a>(
    mut tokens:  impl Iterator<Item = &'a str>,
    line_number: usize,
) -> Result<Vec3, ObjLoadError> {
    let parse_f32 = |s: Option<&str>, component: &str| -> Result<f32, ObjLoadError> {
        s.ok_or_else(|| ObjLoadError::ParseError {
            line_number,
            message: format!("missing {} component", component),
        })?
        .parse::<f32>()
        .map_err(|_| ObjLoadError::ParseError {
            line_number,
            message: format!("invalid float for {} component", component),
        })
    };

    let x = parse_f32(tokens.next(), "x")?;
    let y = parse_f32(tokens.next(), "y")?;
    let z = parse_f32(tokens.next(), "z")?;

    Ok(Vec3::new(x, y, z))
}