// Camera system - built from scratch!

use crate::math::{Vec3, Mat4, deg_to_rad};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    
    // FPS camera controls
    pub yaw: f32,   // Left-right rotation
    pub pitch: f32, // Up-down rotation
    pub forward: Vec3,
    pub right: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, fov: f32, aspect: f32) -> Self {
        let forward = (target - position).normalize();
        let right = forward.cross(&Vec3::Y).normalize();
        
        // Calculate initial yaw and pitch from forward direction
        let yaw = forward.z.atan2(forward.x);
        let pitch = forward.y.asin();
        
        Self {
            position,
            target,
            up: Vec3::Y,
            fov,
            aspect,
            near: 0.1,
            far: 1000.0,
            yaw,
            pitch,
            forward,
            right,
        }
    }

    pub fn perspective() -> Self {
        Self::new(
            Vec3::new(0.0, 2.0, 5.0),
            Vec3::ZERO,
            deg_to_rad(45.0),
            16.0 / 9.0,
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(&self.position, &self.target, &self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        // Perspective matrix with proper aspect ratio handling
        // FOV is vertical, aspect ratio adjusts horizontal FOV
        // This is the CORRECT way - prevents stretching!
        Mat4::perspective(self.fov, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix().mul(&self.view_matrix())
    }

    // FPS-style rotation (mouse look)
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;
        
        // Clamp pitch to prevent flipping
        self.pitch = self.pitch.clamp(-1.5, 1.5);
        
        // Update forward direction
        self.forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();
        
        // Update right direction
        self.right = self.forward.cross(&Vec3::Y).normalize();
        
        // Update target (look at point)
        self.target = self.position + self.forward;
    }

    // Move forward/backward
    pub fn move_forward(&mut self, amount: f32) {
        self.position += self.forward * amount;
        self.target = self.position + self.forward;
    }

    // Move left/right
    pub fn move_right(&mut self, amount: f32) {
        self.position += self.right * amount;
        self.target = self.position + self.forward;
    }

    // Move up/down
    pub fn move_up(&mut self, amount: f32) {
        self.position.y += amount;
        self.target = self.position + self.forward;
    }

    // Legacy orbit method (for backwards compatibility)
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.rotate(delta_yaw, delta_pitch);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.move_forward(-delta);
    }
}

