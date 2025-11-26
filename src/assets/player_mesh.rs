// Player capsule (pill shape) mesh - built from scratch!

use crate::renderer::Vertex;
use crate::math::PI;

pub struct PlayerMesh;

impl PlayerMesh {
    /// Create a capsule (pill) mesh - cylinder with hemisphere caps
    pub fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let half_height = height / 2.0;
        let player_color = [0.2, 0.5, 0.8, 1.0]; // Blue player
        
        // Top hemisphere
        for ring in 0..=rings {
            let phi = (ring as f32 / rings as f32) * (PI / 2.0); // 0 to PI/2
            let y = half_height + radius * phi.sin();
            let ring_radius = radius * phi.cos();
            
            for seg in 0..=segments {
                let theta = (seg as f32 / segments as f32) * PI * 2.0;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();
                
                let normal_x = phi.cos() * theta.cos();
                let normal_y = phi.sin();
                let normal_z = phi.cos() * theta.sin();
                
                vertices.push(Vertex::new(
                    [x, y, z],
                    [normal_x, normal_y, normal_z],
                    [seg as f32 / segments as f32, ring as f32 / rings as f32],
                    player_color,
                ));
            }
        }
        
        // Cylinder middle
        let cylinder_start = vertices.len() as u16;
        for i in 0..=1 {
            let y = if i == 0 { half_height } else { -half_height };
            
            for seg in 0..=segments {
                let theta = (seg as f32 / segments as f32) * PI * 2.0;
                let x = radius * theta.cos();
                let z = radius * theta.sin();
                
                vertices.push(Vertex::new(
                    [x, y, z],
                    [theta.cos(), 0.0, theta.sin()],
                    [seg as f32 / segments as f32, i as f32],
                    player_color,
                ));
            }
        }
        
        // Bottom hemisphere
        let bottom_start = vertices.len() as u16;
        for ring in 0..=rings {
            let phi = (ring as f32 / rings as f32) * (PI / 2.0);
            let y = -half_height - radius * phi.sin();
            let ring_radius = radius * phi.cos();
            
            for seg in 0..=segments {
                let theta = (seg as f32 / segments as f32) * PI * 2.0;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();
                
                let normal_x = phi.cos() * theta.cos();
                let normal_y = -phi.sin();
                let normal_z = phi.cos() * theta.sin();
                
                vertices.push(Vertex::new(
                    [x, y, z],
                    [normal_x, normal_y, normal_z],
                    [seg as f32 / segments as f32, ring as f32 / rings as f32],
                    player_color,
                ));
            }
        }
        
        // Generate indices for top hemisphere (CCW winding for front faces)
        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;
                
                // Triangle 1 - CCW winding (visible from outside)
                indices.push(current as u16);
                indices.push(next as u16);
                indices.push((current + 1) as u16);
                
                // Triangle 2 - CCW winding (visible from outside)
                indices.push((current + 1) as u16);
                indices.push(next as u16);
                indices.push((next + 1) as u16);
            }
        }
        
        // Generate indices for cylinder (current=top row, next=bottom row)
        for seg in 0..segments {
            let current = cylinder_start + seg as u16;
            let next = current + (segments + 1) as u16;
            
            // Triangle 1 - CCW from outside: top-left -> top-right -> bottom-left
            indices.push(current);
            indices.push(current + 1);
            indices.push(next);
            
            // Triangle 2 - CCW from outside: top-right -> bottom-right -> bottom-left
            indices.push(current + 1);
            indices.push(next + 1);
            indices.push(next);
        }
        
        // Generate indices for bottom hemisphere (builds from equator DOWN to pole)
        // Winding must account for downward-facing triangles
        for ring in 0..rings {
            for seg in 0..segments {
                let current = bottom_start + (ring * (segments + 1) + seg) as u16;
                let next = current + (segments + 1) as u16;
                
                // Triangle 1 - Flip winding for bottom hemisphere
                indices.push(next as u16);
                indices.push(current as u16);
                indices.push((current + 1) as u16);
                
                // Triangle 2
                indices.push(next as u16);
                indices.push((current + 1) as u16);
                indices.push((next + 1) as u16);
            }
        }
        
        (vertices, indices)
    }
}

