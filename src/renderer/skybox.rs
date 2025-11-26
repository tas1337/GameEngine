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
            Vertex::new([-size,  size, -size], [0.0, 1.0, 0.0], [0.0, 0.0], [0.65, 0.82, 1.05, 1.0]),
            Vertex::new([ size,  size, -size], [0.0, 1.0, 0.0], [1.0, 0.0], [0.65, 0.82, 1.05, 1.0]),
            Vertex::new([ size,  size,  size], [0.0, 1.0, 0.0], [1.0, 1.0], [0.65, 0.82, 1.05, 1.0]),
            Vertex::new([-size,  size,  size], [0.0, 1.0, 0.0], [0.0, 1.0], [0.65, 0.82, 1.05, 1.0]),
            
            // Sides (gradient from bottom to top)
            Vertex::new([-size, -size, -size], [-1.0, 0.0, 0.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size, -size,  size], [-1.0, 0.0, 0.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size,  size,  size], [-1.0, 0.0, 0.0], [1.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            Vertex::new([-size,  size, -size], [-1.0, 0.0, 0.0], [0.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            
            Vertex::new([ size, -size,  size], [1.0, 0.0, 0.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size, -size, -size], [1.0, 0.0, 0.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size,  size, -size], [1.0, 0.0, 0.0], [1.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            Vertex::new([ size,  size,  size], [1.0, 0.0, 0.0], [0.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            
            Vertex::new([-size, -size, -size], [0.0, 0.0, -1.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size, -size, -size], [0.0, 0.0, -1.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([ size,  size, -size], [0.0, 0.0, -1.0], [1.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            Vertex::new([-size,  size, -size], [0.0, 0.0, -1.0], [0.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            
            Vertex::new([ size, -size,  size], [0.0, 0.0, 1.0], [0.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size, -size,  size], [0.0, 0.0, 1.0], [1.0, 0.0], [0.2, 0.3, 0.4, 1.0]),
            Vertex::new([-size,  size,  size], [0.0, 0.0, 1.0], [1.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
            Vertex::new([ size,  size,  size], [0.0, 0.0, 1.0], [0.0, 1.0], [0.55, 0.75, 1.0, 1.0]),
        ];

        let vertex_data = super::vertex_data_interleaved(&vertices);
        let vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        vbo.set_data(gl, &vertex_data, GL::STATIC_DRAW);
        
        let index_count = 36; // 6 faces × 2 triangles × 3 vertices

        Ok(Self {
            vbo,
            index_count,
            time_of_day: 0.25, // Start at dawn
            cycle_speed: 0.015,  // Slower cycle (~70 seconds locally)
        })
    }

    pub fn update(&mut self, dt: f32) {
        self.time_of_day += self.cycle_speed * dt;
        if self.time_of_day > 1.0 {
            self.time_of_day -= 1.0;
        }
    }

    pub fn get_sky_color(&self) -> Vec3 {
        let t = self.time_of_day;
        
        let night_color = Vec3::new(0.03, 0.05, 0.15);
        let dawn_color = Vec3::new(0.92, 0.63, 0.42);
        let day_color = Vec3::new(0.85, 0.97, 1.3);
        let dusk_color = Vec3::new(0.96, 0.55, 0.4);
        
        let dawn_weight = Self::smooth_time_weight(t, 0.18, 0.38);
        let mut day_weight = Self::smooth_time_weight(t, 0.3, 0.9);
        day_weight = day_weight.powf(0.8);  // Keep midday bright but less contrast
        let dusk_weight = Self::smooth_time_weight(t, 0.8, 1.05);
        let mut night_weight = 1.0 - (dawn_weight + day_weight + dusk_weight);
        if night_weight < 0.0 {
            night_weight = 0.0;
        }
        
        let total = night_weight + dawn_weight + day_weight + dusk_weight;
        let safe_total = total.max(0.0001);
        
        let mut color = night_color * night_weight;
        color += dawn_color * dawn_weight;
        color += day_color * day_weight;
        color += dusk_color * dusk_weight;
        
        color / safe_total
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

    fn smooth_time_weight(time: f32, start: f32, end: f32) -> f32 {
        let mut s = start;
        let mut e = end;
        let mut t = time;
        
        if e < s {
            e += 1.0;
            if t < s {
                t += 1.0;
            }
        }
        
        if t < s || t > e {
            return 0.0;
        }
        
        let normalized = (t - s) / (e - s);
        let peak = 1.0 - (normalized * 2.0 - 1.0).abs(); // Triangle wave 0..1..0
        peak * peak  // Ease at the top
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

