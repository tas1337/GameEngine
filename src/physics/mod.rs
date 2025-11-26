// Physics system - built from scratch!

pub mod collision;
pub mod rigidbody;

pub use collision::*;
pub use rigidbody::*;

use crate::math::Vec3;

pub const GRAVITY: Vec3 = Vec3 { x: 0.0, y: -9.8, z: 0.0 };

/// Simple AABB (Axis-Aligned Bounding Box)
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn contains_point(&self, point: &Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

/// Plane for ground collision
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let distance = normal.dot(&point);
        Self { normal, distance }
    }

    /// Distance from point to plane (negative = below)
    pub fn distance_to_point(&self, point: &Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}

