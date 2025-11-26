// Vertex structure - built from scratch!

/// Vertex layout for a simple 3D vertex
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            normal,
            uv,
            color,
        }
    }
}
