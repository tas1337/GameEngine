// Component system - built from scratch!

use crate::math::{Vec3, Quat};
use serde::{Serialize, Deserialize};

/// Transform component - position, rotation, scale
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn model_matrix(&self) -> crate::math::Mat4 {
        let translation = crate::math::Mat4::translate(self.position.x, self.position.y, self.position.z);
        let rotation = self.rotation.to_mat4();
        let scale = crate::math::Mat4::scale(self.scale.x, self.scale.y, self.scale.z);
        translation.mul(&rotation).mul(&scale)
    }
}

/// Velocity component for physics
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn new() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }

    pub fn with_linear(mut self, linear: Vec3) -> Self {
        self.linear = linear;
        self
    }
}

/// Color component
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn as_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Lifetime component for particles
#[derive(Debug, Clone, Copy)]
pub struct Lifetime {
    pub remaining: f32,
    pub total: f32,
}

impl Lifetime {
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
            total: duration,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.remaining > 0.0
    }

    pub fn progress(&self) -> f32 {
        1.0 - (self.remaining / self.total)
    }
}

