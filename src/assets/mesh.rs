// Mesh generation - built from scratch!

use crate::renderer::Vertex;
use crate::math::{Vec3, PI};

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self { vertices, indices }
    }

    /// Generate a cube mesh
    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            // Front face
            Vertex::new([-s, -s,  s], [ 0.0,  0.0,  1.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s, -s,  s], [ 0.0,  0.0,  1.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s,  s], [ 0.0,  0.0,  1.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s,  s], [ 0.0,  0.0,  1.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            
            // Back face
            Vertex::new([ s, -s, -s], [ 0.0,  0.0, -1.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s, -s, -s], [ 0.0,  0.0, -1.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s, -s], [ 0.0,  0.0, -1.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s, -s], [ 0.0,  0.0, -1.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            
            // Top face
            Vertex::new([-s,  s,  s], [ 0.0,  1.0,  0.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s,  s], [ 0.0,  1.0,  0.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s, -s], [ 0.0,  1.0,  0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s, -s], [ 0.0,  1.0,  0.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            
            // Bottom face
            Vertex::new([-s, -s, -s], [ 0.0, -1.0,  0.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s, -s, -s], [ 0.0, -1.0,  0.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s, -s,  s], [ 0.0, -1.0,  0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s, -s,  s], [ 0.0, -1.0,  0.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            
            // Right face
            Vertex::new([ s, -s,  s], [ 1.0,  0.0,  0.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s, -s, -s], [ 1.0,  0.0,  0.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s, -s], [ 1.0,  0.0,  0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s,  s], [ 1.0,  0.0,  0.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            
            // Left face
            Vertex::new([-s, -s, -s], [-1.0,  0.0,  0.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s, -s,  s], [-1.0,  0.0,  0.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s,  s], [-1.0,  0.0,  0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s, -s], [-1.0,  0.0,  0.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ];

        let indices = vec![
            0, 1, 2,  0, 2, 3,    // Front
            4, 5, 6,  4, 6, 7,    // Back
            8, 9, 10, 8, 10, 11,  // Top
            12, 13, 14, 12, 14, 15, // Bottom
            16, 17, 18, 16, 18, 19, // Right
            20, 21, 22, 20, 22, 23, // Left
        ];

        Self::new(vertices, indices)
    }

    /// Generate a sphere mesh
    pub fn sphere(radius: f32, stacks: u32, slices: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=stacks {
            let v = i as f32 / stacks as f32;
            let phi = v * PI;

            for j in 0..=slices {
                let u = j as f32 / slices as f32;
                let theta = u * PI * 2.0;

                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();

                let normal = Vec3::new(x, y, z).normalize();

                vertices.push(Vertex::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    [u, v],
                    [1.0, 1.0, 1.0, 1.0],
                ));
            }
        }

        // Generate indices
        for i in 0..stacks {
            for j in 0..slices {
                let first = (i * (slices + 1) + j) as u16;
                let second = first + slices as u16 + 1;

                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        Self::new(vertices, indices)
    }

    /// Generate a simple quad (for particles)
    pub fn quad(size: f32) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            Vertex::new([-s, -s, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s, -s, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([ s,  s, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-s,  s, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        Self::new(vertices, indices)
    }
}

