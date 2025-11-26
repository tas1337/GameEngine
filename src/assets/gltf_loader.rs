// GLTF Loader - built from scratch in pure Rust!
// Parses GLTF JSON and binary data at compile time

use crate::renderer::Vertex;
use crate::assets::Mesh;

// Embed the GLTF files at compile time
const GLTF_JSON: &str = include_str!("gbl/cube/scene.gltf");
const GLTF_BIN: &[u8] = include_bytes!("gbl/cube/scene.bin");

/// Load the Companion Cube mesh from embedded GLTF data
pub fn companion_cube() -> Mesh {
    // Parse GLTF JSON manually (no serde needed for this simple case)
    let gltf = parse_gltf_json(GLTF_JSON);
    
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut index_offset = 0u16;
    
    // Scale factor: GLTF model is ~1.82 units wide, we want it to be 1 unit (like original cube)
    // So we scale by 1.0 / 1.82 ≈ 0.55 to match original box collision size
    let scale = 0.55;
    
    // Process each mesh and primitive
    for mesh_idx in &gltf.mesh_indices {
        let mesh = &gltf.meshes[*mesh_idx];
        
        // Get accessor data
        let pos_accessor = &gltf.accessors[mesh.position_accessor];
        let norm_accessor = &gltf.accessors[mesh.normal_accessor];
        let uv_accessor = &gltf.accessors[mesh.uv_accessor];
        let idx_accessor = &gltf.accessors[mesh.index_accessor];
        
        // Read vertex data from binary
        let positions = read_vec3s(GLTF_BIN, pos_accessor, &gltf.buffer_views);
        let normals = read_vec3s(GLTF_BIN, norm_accessor, &gltf.buffer_views);
        let uvs = read_vec2s(GLTF_BIN, uv_accessor, &gltf.buffer_views);
        let indices = read_indices(GLTF_BIN, idx_accessor, &gltf.buffer_views);
        
        // Create vertices with scaled positions
        // Use Portal-style colors: gray base with pink heart accent
        for i in 0..positions.len() {
            // Use UV coordinates to determine color (heart area vs body)
            let u = uvs[i].0;
            let v = uvs[i].1;
            
            // Pink accent for heart areas (center of texture), gray for body
            let is_heart = u > 0.3 && u < 0.7 && v > 0.3 && v < 0.7;
            let color = if is_heart {
                [0.95, 0.4, 0.6, 1.0]  // Pink heart
            } else {
                [0.6, 0.6, 0.65, 1.0]  // Gray body
            };
            
            all_vertices.push(Vertex::new(
                [positions[i].0 * scale, positions[i].1 * scale, positions[i].2 * scale],
                [normals[i].0, normals[i].1, normals[i].2],
                [uvs[i].0, uvs[i].1],
                color,
            ));
        }
        
        // Add indices with offset
        for idx in indices {
            all_indices.push(idx as u16 + index_offset);
        }
        
        index_offset += positions.len() as u16;
    }
    
    Mesh::new(all_vertices, all_indices)
}

// Simple GLTF structure
struct GltfData {
    accessors: Vec<Accessor>,
    buffer_views: Vec<BufferView>,
    meshes: Vec<MeshPrimitive>,
    mesh_indices: Vec<usize>,
}

struct Accessor {
    buffer_view: usize,
    byte_offset: usize,
    count: usize,
    component_type: u32,  // 5126 = float, 5125 = u32, 5123 = u16
}

struct BufferView {
    byte_offset: usize,
    byte_length: usize,
    byte_stride: usize,
}

struct MeshPrimitive {
    position_accessor: usize,
    normal_accessor: usize,
    uv_accessor: usize,
    index_accessor: usize,
}

/// Parse GLTF JSON manually (simple parser for our specific file)
fn parse_gltf_json(json: &str) -> GltfData {
    // Find accessors array
    let accessors = parse_accessors(json);
    let buffer_views = parse_buffer_views(json);
    let (meshes, mesh_indices) = parse_meshes(json);
    
    GltfData {
        accessors,
        buffer_views,
        meshes,
        mesh_indices,
    }
}

fn parse_accessors(json: &str) -> Vec<Accessor> {
    let mut accessors = Vec::new();
    
    // Find "accessors": [ ... ]
    if let Some(start) = json.find("\"accessors\"") {
        let rest = &json[start..];
        if let Some(arr_start) = rest.find('[') {
            let arr_rest = &rest[arr_start..];
            
            // Parse each accessor object
            let mut depth = 0;
            let mut obj_start = 0;
            let mut in_obj = false;
            
            for (i, c) in arr_rest.chars().enumerate() {
                match c {
                    '[' if depth == 0 => depth = 1,
                    '{' => {
                        if depth == 1 && !in_obj {
                            obj_start = i;
                            in_obj = true;
                        }
                        depth += 1;
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 1 && in_obj {
                            let obj = &arr_rest[obj_start..=i];
                            accessors.push(parse_accessor_obj(obj));
                            in_obj = false;
                        }
                    }
                    ']' if depth == 1 => break,
                    _ => {}
                }
            }
        }
    }
    
    accessors
}

fn parse_accessor_obj(obj: &str) -> Accessor {
    Accessor {
        buffer_view: find_int(obj, "bufferView").unwrap_or(0),
        byte_offset: find_int(obj, "byteOffset").unwrap_or(0),
        count: find_int(obj, "count").unwrap_or(0),
        component_type: find_int(obj, "componentType").unwrap_or(5126) as u32,
    }
}

fn parse_buffer_views(json: &str) -> Vec<BufferView> {
    let mut views = Vec::new();
    
    if let Some(start) = json.find("\"bufferViews\"") {
        let rest = &json[start..];
        if let Some(arr_start) = rest.find('[') {
            let arr_rest = &rest[arr_start..];
            
            let mut depth = 0;
            let mut obj_start = 0;
            let mut in_obj = false;
            
            for (i, c) in arr_rest.chars().enumerate() {
                match c {
                    '[' if depth == 0 => depth = 1,
                    '{' => {
                        if depth == 1 && !in_obj {
                            obj_start = i;
                            in_obj = true;
                        }
                        depth += 1;
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 1 && in_obj {
                            let obj = &arr_rest[obj_start..=i];
                            views.push(BufferView {
                                byte_offset: find_int(obj, "byteOffset").unwrap_or(0),
                                byte_length: find_int(obj, "byteLength").unwrap_or(0),
                                byte_stride: find_int(obj, "byteStride").unwrap_or(0),
                            });
                            in_obj = false;
                        }
                    }
                    ']' if depth == 1 => break,
                    _ => {}
                }
            }
        }
    }
    
    views
}

fn parse_meshes(json: &str) -> (Vec<MeshPrimitive>, Vec<usize>) {
    let mut meshes = Vec::new();
    let mut indices = Vec::new();
    
    if let Some(start) = json.find("\"meshes\"") {
        let rest = &json[start..];
        if let Some(arr_start) = rest.find('[') {
            let arr_rest = &rest[arr_start..];
            
            let mut depth = 0;
            let mut obj_start = 0;
            let mut in_obj = false;
            let mut mesh_idx = 0;
            
            for (i, c) in arr_rest.chars().enumerate() {
                match c {
                    '[' if depth == 0 => depth = 1,
                    '{' => {
                        if depth == 1 && !in_obj {
                            obj_start = i;
                            in_obj = true;
                        }
                        depth += 1;
                    }
                    '}' => {
                        depth -= 1;
                        if depth == 1 && in_obj {
                            let obj = &arr_rest[obj_start..=i];
                            // Find primitives in this mesh
                            if let Some(prim) = parse_primitive(obj) {
                                indices.push(meshes.len());
                                meshes.push(prim);
                            }
                            in_obj = false;
                            mesh_idx += 1;
                        }
                    }
                    ']' if depth == 1 => break,
                    _ => {}
                }
            }
        }
    }
    
    (meshes, indices)
}

fn parse_primitive(mesh_obj: &str) -> Option<MeshPrimitive> {
    // Find attributes
    let position = find_int(mesh_obj, "\"POSITION\"")?;
    let normal = find_int(mesh_obj, "\"NORMAL\"")?;
    let uv = find_int(mesh_obj, "\"TEXCOORD_0\"")?;
    let indices = find_int(mesh_obj, "\"indices\"")?;
    
    Some(MeshPrimitive {
        position_accessor: position,
        normal_accessor: normal,
        uv_accessor: uv,
        index_accessor: indices,
    })
}

fn find_int(s: &str, key: &str) -> Option<usize> {
    let key_pos = s.find(key)?;
    let rest = &s[key_pos + key.len()..];
    
    // Skip to colon and whitespace
    let colon_pos = rest.find(':')?;
    let after_colon = &rest[colon_pos + 1..];
    
    // Find the number
    let trimmed = after_colon.trim_start();
    let mut num_str = String::new();
    
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            num_str.push(c);
        } else if !num_str.is_empty() {
            break;
        }
    }
    
    num_str.parse().ok()
}

/// Read Vec3 data from binary buffer
fn read_vec3s(bin: &[u8], accessor: &Accessor, views: &[BufferView]) -> Vec<(f32, f32, f32)> {
    let view = &views[accessor.buffer_view];
    let base_offset = view.byte_offset + accessor.byte_offset;
    let stride = if view.byte_stride > 0 { view.byte_stride } else { 12 };  // 3 floats = 12 bytes
    
    let mut result = Vec::with_capacity(accessor.count);
    
    for i in 0..accessor.count {
        let offset = base_offset + i * stride;
        let x = read_f32(bin, offset);
        let y = read_f32(bin, offset + 4);
        let z = read_f32(bin, offset + 8);
        result.push((x, y, z));
    }
    
    result
}

/// Read Vec2 data from binary buffer
fn read_vec2s(bin: &[u8], accessor: &Accessor, views: &[BufferView]) -> Vec<(f32, f32)> {
    let view = &views[accessor.buffer_view];
    let base_offset = view.byte_offset + accessor.byte_offset;
    let stride = if view.byte_stride > 0 { view.byte_stride } else { 8 };  // 2 floats = 8 bytes
    
    let mut result = Vec::with_capacity(accessor.count);
    
    for i in 0..accessor.count {
        let offset = base_offset + i * stride;
        let x = read_f32(bin, offset);
        let y = read_f32(bin, offset + 4);
        result.push((x, y));
    }
    
    result
}

/// Read index data from binary buffer
fn read_indices(bin: &[u8], accessor: &Accessor, views: &[BufferView]) -> Vec<u32> {
    let view = &views[accessor.buffer_view];
    let base_offset = view.byte_offset + accessor.byte_offset;
    
    let mut result = Vec::with_capacity(accessor.count);
    
    // Component type: 5125 = u32, 5123 = u16
    let bytes_per_index = if accessor.component_type == 5125 { 4 } else { 2 };
    
    for i in 0..accessor.count {
        let offset = base_offset + i * bytes_per_index;
        let idx = if bytes_per_index == 4 {
            read_u32(bin, offset)
        } else {
            read_u16(bin, offset) as u32
        };
        result.push(idx);
    }
    
    result
}

#[inline]
fn read_f32(data: &[u8], offset: usize) -> f32 {
    let bytes = [data[offset], data[offset+1], data[offset+2], data[offset+3]];
    f32::from_le_bytes(bytes)
}

#[inline]
fn read_u32(data: &[u8], offset: usize) -> u32 {
    let bytes = [data[offset], data[offset+1], data[offset+2], data[offset+3]];
    u32::from_le_bytes(bytes)
}

#[inline]
fn read_u16(data: &[u8], offset: usize) -> u16 {
    let bytes = [data[offset], data[offset+1]];
    u16::from_le_bytes(bytes)
}

