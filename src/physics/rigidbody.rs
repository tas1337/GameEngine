// Rigidbody physics - built from scratch!

use crate::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Rigidbody {
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub mass: f32,
    pub drag: f32,
    pub use_gravity: bool,
    pub is_grounded: bool,
}

impl Rigidbody {
    pub fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0,
            drag: 0.98,
            use_gravity: true,
            is_grounded: false,
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.acceleration += force / self.mass;
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse / self.mass;
    }

    pub fn update(&mut self, dt: f32, gravity: Vec3) {
        // Apply gravity
        if self.use_gravity && !self.is_grounded {
            self.acceleration += gravity;
        }

        // Update velocity
        self.velocity += self.acceleration * dt;

        // Apply drag
        self.velocity *= self.drag;

        // Reset acceleration
        self.acceleration = Vec3::ZERO;
    }

    pub fn get_displacement(&self, dt: f32) -> Vec3 {
        self.velocity * dt
    }
}

