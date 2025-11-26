// Collision detection - built from scratch!

use crate::math::Vec3;
use super::{AABB, Plane};

/// Ray for raycasting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Ray-plane intersection
pub fn ray_plane_intersection(ray: &Ray, plane: &Plane) -> Option<f32> {
    let denom = plane.normal.dot(&ray.direction);
    
    if denom.abs() < 0.0001 {
        return None; // Parallel
    }

    let t = (plane.distance - plane.normal.dot(&ray.origin)) / denom;
    
    if t >= 0.0 {
        Some(t)
    } else {
        None
    }
}

/// Sphere-plane collision
pub fn sphere_plane_collision(center: &Vec3, radius: f32, plane: &Plane) -> Option<Vec3> {
    let dist = plane.distance_to_point(center);
    
    if dist < radius {
        // Collision! Push sphere up
        let penetration = radius - dist;
        Some(plane.normal * penetration)
    } else {
        None
    }
}

/// AABB-AABB collision
pub fn aabb_collision(a: &AABB, b: &AABB) -> Option<Vec3> {
    if !a.intersects(b) {
        return None;
    }

    // Calculate overlap on each axis
    let overlap_x = (a.max.x - b.min.x).min(b.max.x - a.min.x);
    let overlap_y = (a.max.y - b.min.y).min(b.max.y - a.min.y);
    let overlap_z = (a.max.z - b.min.z).min(b.max.z - a.min.z);

    // Find minimum overlap axis (MTV - Minimum Translation Vector)
    if overlap_x < overlap_y && overlap_x < overlap_z {
        Some(Vec3::new(overlap_x * if a.min.x < b.min.x { -1.0 } else { 1.0 }, 0.0, 0.0))
    } else if overlap_y < overlap_z {
        Some(Vec3::new(0.0, overlap_y * if a.min.y < b.min.y { -1.0 } else { 1.0 }, 0.0))
    } else {
        Some(Vec3::new(0.0, 0.0, overlap_z * if a.min.z < b.min.z { -1.0 } else { 1.0 }))
    }
}

