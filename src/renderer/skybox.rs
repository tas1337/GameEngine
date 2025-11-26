// Skybox system - built from scratch!

use crate::math::{Vec3, Mat4};
use crate::renderer::{Vertex, GLBuffer};
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;

pub struct Skybox {
    pub vbo: GLBuffer,
    pub index_count: i32,
    pub time_of_day: f32,  // 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    pub cycle_speed: f32,   // How fast day/night cycles
}

impl Skybox {
    pub fn new(gl: &web_sys::WebGl2RenderingContext) -> Result<Self, JsValue> {
        // Create a large inverted cube for skybox
        let size = 500.0;
        
        let vertices = vec![
            // We'll render this as a gradient, so normals point inward
            // Bottom (dark)
            Vertex::new([-size, -size, -size], [0.0, -1.0, 0.0], [0.0, 0.0], [0.1, 0.1, 0.2, 1.0]),
            Vertex::new([ size, -size, -size], [0.0, -1.0, 0.0], [1.0, 0.0], [0.1, 0.1, 0.2, 1.0]),
            Vertex::new([ size, -size,  size], [0.0, -1.0, 0.0], [1.0, 1.0], [0.1, 0.1, 0.2, 1.0]),
            Vertex::new([-size, -size,  size], [0.0, -1.0, 0.0], [0.0, 1.0], [0.1, 0.1, 0.2, 1.0]),
            
            // Top (bright)
            Vertex::new([-size,  size, -size], [0.0, 1.0, 0.0], [0.0, 0.0], [0.5, 0.7, 1.0, 1.0]),
            Vertex::new([ size,  size, -size], [0.0, 1.0, 0.0], [1.0, 0.0], [0.5, 0.7, 1.0, 1.0]),
            Vertex::new([ size,  size,  size], [0.0, 1.0, 0.0], [1.0, 1.0], [0.5, 0.7, 1.0, 1.0]),
            Vertex::new([-size,  size,  size], [0.0, 1.0, 0.0], [0.0, 1.0], [0.5, 0.7, 1.0, 1.0]),
            
            // Sides (gradient from bottom to top)
            Vertex::new([-size, -size, -size], [-1.0, 0.0, 0.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size, -size,  size], [-1.0, 0.0, 0.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size,  size,  size], [-1.0, 0.0, 0.0], [1.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            Vertex::new([-size,  size, -size], [-1.0, 0.0, 0.0], [0.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            
            Vertex::new([ size, -size,  size], [1.0, 0.0, 0.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size, -size, -size], [1.0, 0.0, 0.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size,  size, -size], [1.0, 0.0, 0.0], [1.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            Vertex::new([ size,  size,  size], [1.0, 0.0, 0.0], [0.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            
            Vertex::new([-size, -size, -size], [0.0, 0.0, -1.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size, -size, -size], [0.0, 0.0, -1.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size,  size, -size], [0.0, 0.0, -1.0], [1.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            Vertex::new([-size,  size, -size], [0.0, 0.0, -1.0], [0.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            
            Vertex::new([ size, -size,  size], [0.0, 0.0, 1.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size, -size,  size], [0.0, 0.0, 1.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size,  size,  size], [0.0, 0.0, 1.0], [1.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
            Vertex::new([ size,  size,  size], [0.0, 0.0, 1.0], [0.0, 1.0], [0.4, 0.6, 0.9, 1.0]),
        ];

        let vertex_data = super::vertex_data_interleaved(&vertices);
        let vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        vbo.set_data(gl, &vertex_data, GL::STATIC_DRAW);
        
        let index_count = 36; // 6 faces × 2 triangles × 3 vertices

        Ok(Self {
            vbo,
            index_count,
            time_of_day: 0.25, // Start at dawn
            cycle_speed: 0.05,  // 0.05 = ~20 second full cycle
        })
    }

    pub fn update(&mut self, dt: f32) {
        self.time_of_day += self.cycle_speed * dt;
        if self.time_of_day > 1.0 {
            self.time_of_day -= 1.0;
        }
    }

    pub fn get_sky_color(&self) -> Vec3 {
        // Day/night cycle colors
        let t = self.time_of_day;
        
        if t < 0.25 {
            // Night to dawn (dark blue to orange)
            let factor = t * 4.0;
            Vec3::new(
                0.1 + factor * 0.9,
                0.1 + factor * 0.4,
                0.2 + factor * 0.3,
            )
        } else if t < 0.5 {
            // Dawn to noon (orange to bright blue)
            let factor = (t - 0.25) * 4.0;
            Vec3::new(
                1.0 - factor * 0.5,
                0.5 + factor * 0.3,
                0.5 + factor * 0.5,
            )
        } else if t < 0.75 {
            // Noon to dusk (bright blue to orange)
            let factor = (t - 0.5) * 4.0;
            Vec3::new(
                0.5 + factor * 0.5,
                0.8 - factor * 0.3,
                1.0 - factor * 0.5,
            )
        } else {
            // Dusk to night (orange to dark blue)
            let factor = (t - 0.75) * 4.0;
            Vec3::new(
                1.0 - factor * 0.9,
                0.5 - factor * 0.4,
                0.5 - factor * 0.3,
            )
        }
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day < 0.2 || self.time_of_day > 0.8
    }

    pub fn get_sun_direction(&self) -> Vec3 {
        // Sun rotates from east to west
        let angle = self.time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI / 2.0;
        Vec3::new(
            angle.cos(),
            angle.sin(),
            0.2,
        ).normalize()
    }
}

