// Sun mesh - built from scratch!

use crate::renderer::Vertex;
use crate::math::PI;

pub struct SunMesh;

impl SunMesh {
    /// Create a sun sphere (bright yellow circle)
    pub fn sun(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let yellow = [1.0, 0.9, 0.3, 1.0];  // Bright yellow sun
        
        // Create UV sphere
        for i in 0..=segments {
            let v = i as f32 / segments as f32;
            let phi = v * PI;

            for j in 0..=segments {
                let u = j as f32 / segments as f32;
                let theta = u * PI * 2.0;

                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();

                let normal = crate::math::Vec3::new(x, y, z).normalize();

                vertices.push(Vertex::new(
                    [x, y, z],
                    [normal.x, normal.y, normal.z],
                    [u, v],
                    yellow,
                ));
            }
        }

        // Generate indices
        for i in 0..segments {
            for j in 0..segments {
                let first = (i * (segments + 1) + j) as u16;
                let second = first + (segments + 1) as u16;

                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        (vertices, indices)
    }
}

