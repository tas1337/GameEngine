// Web Workers - Multi-threaded physics from scratch!

use wasm_bindgen::prelude::*;
use crate::math::Vec3;
use crate::physics::GRAVITY;

/// Physics update data that can be sent to/from workers
#[derive(Clone, Copy, Debug)]
pub struct PhysicsData {
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
}

// Internal methods (not exposed to JS)
impl PhysicsData {
    pub fn new(
        position_x: f32, position_y: f32, position_z: f32,
        velocity_x: f32, velocity_y: f32, velocity_z: f32,
    ) -> Self {
        Self {
            position_x, position_y, position_z,
            velocity_x, velocity_y, velocity_z,
        }
    }

    pub fn position(&self) -> Vec3 {
        Vec3::new(self.position_x, self.position_y, self.position_z)
    }

    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.velocity_x, self.velocity_y, self.velocity_z)
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position_x = pos.x;
        self.position_y = pos.y;
        self.position_z = pos.z;
    }

    pub fn set_velocity(&mut self, vel: Vec3) {
        self.velocity_x = vel.x;
        self.velocity_y = vel.y;
        self.velocity_z = vel.z;
    }
}

/// Update physics for a batch of particles (internal use only)
pub fn update_physics_batch(data: &mut [PhysicsData], dt: f32) {
    for particle in data.iter_mut() {
        let mut pos = particle.position();
        let mut vel = particle.velocity();
        
        // Apply gravity
        vel += GRAVITY * dt;
        
        // Update position
        pos += vel * dt;
        
        // Ground collision
        if pos.y < 0.0 {
            pos.y = 0.0;
            vel.y = -vel.y * 0.6;  // Bounce
            vel.x *= 0.95;         // Friction
            vel.z *= 0.95;
        }
        
        particle.set_position(pos);
        particle.set_velocity(vel);
    }
}

/// Worker pool manager
pub struct WorkerPool {
    pub worker_count: usize,
    pub enabled: bool,
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> Self {
        Self {
            worker_count,
            enabled: false,  // Disabled by default until we implement workers
        }
    }

    pub fn optimal_worker_count() -> usize {
        // Try to get hardware concurrency (CPU core count)
        if let Some(window) = web_sys::window() {
            // Get navigator object using Reflect API
            if let Ok(nav_obj) = js_sys::Reflect::get(&window, &"navigator".into()) {
                // Try to get hardwareConcurrency property from navigator
                if let Ok(cores) = js_sys::Reflect::get(&nav_obj, &"hardwareConcurrency".into()) {
                    if let Some(count) = cores.as_f64() {
                        return (count as usize).max(2).min(8);  // 2-8 workers
                    }
                }
            }
        }
        
        4  // Default to 4 workers
    }
}

/// Chunk size for worker batching
pub const CHUNK_SIZE: usize = 10000;  // Each worker processes 10K particles

/// Split particles into chunks for workers
pub fn chunk_indices(total: usize, chunks: usize) -> Vec<(usize, usize)> {
    let chunk_size = (total + chunks - 1) / chunks;
    let mut result = Vec::new();
    
    for i in 0..chunks {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(total);
        
        if start < total {
            result.push((start, end));
        }
    }
    
    result
}

