// Skybox system - built from scratch!

use crate::math::Vec3;
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
        // Smooth day/night cycle using cosine interpolation
        // time: 0.0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset, 1.0 = midnight
        let t = self.time_of_day;
        
        // Calculate sun height using smooth cosine curve (-1 at midnight, 1 at noon)
        let sun_height = (t * std::f32::consts::PI * 2.0).cos() * -1.0;
        
        // Smooth transition factor (0 = night, 1 = day)
        // Use smoothstep for even smoother transitions
        let day_factor = Self::smoothstep(-0.3, 0.3, sun_height);
        
        // Define key colors
        let night_color = Vec3::new(0.02, 0.02, 0.08);     // Deep dark blue
        let dawn_dusk_color = Vec3::new(0.9, 0.4, 0.2);    // Orange/pink
        let day_color = Vec3::new(0.4, 0.7, 1.0);          // Bright blue sky
        
        // Calculate how close we are to dawn/dusk (peaks at sunrise/sunset)
        let dawn_dusk_factor = {
            // Sun near horizon = dawn/dusk
            let horizon_proximity = 1.0 - sun_height.abs();
            // Only show orange in narrow window near actual dawn/dusk times
            let time_factor = if (t > 0.20 && t < 0.30) || (t > 0.70 && t < 0.80) {
                Self::smoothstep(0.0, 0.7, horizon_proximity)
            } else {
                0.0
            };
            time_factor * 0.5  // Max 50% orange blend (less intense)
        };
        
        // Blend between night and day
        let base_color = Self::lerp_vec3(&night_color, &day_color, day_factor);
        
        // Add dawn/dusk orange tint
        Self::lerp_vec3(&base_color, &dawn_dusk_color, dawn_dusk_factor)
    }
    
    /// Smooth interpolation (like GLSL smoothstep)
    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    
    /// Linear interpolation between two Vec3
    fn lerp_vec3(a: &Vec3, b: &Vec3, t: f32) -> Vec3 {
        Vec3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
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

