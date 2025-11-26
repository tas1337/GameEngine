// LOD (Level of Detail) System - built from scratch like Unreal Engine!

use crate::math::Vec3;
use crate::renderer::Camera;

/// LOD Level definition
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LODLevel {
    High = 0,      // Close to camera - full detail
    Medium = 1,    // Medium distance - reduced detail
    Low = 2,       // Far distance - low detail
    VeryLow = 3,   // Very far - minimal detail
    Culled = 4,    // Too far or occluded - don't render
}

impl LODLevel {
    pub fn get_mesh_quality(&self) -> f32 {
        match self {
            LODLevel::High => 1.0,
            LODLevel::Medium => 0.6,
            LODLevel::Low => 0.3,
            LODLevel::VeryLow => 0.1,
            LODLevel::Culled => 0.0,
        }
    }

    pub fn should_render(&self) -> bool {
        *self != LODLevel::Culled
    }
}

/// LOD Configuration
pub struct LODConfig {
    pub distance_high: f32,      // 0 - 20 units
    pub distance_medium: f32,    // 20 - 50 units
    pub distance_low: f32,       // 50 - 100 units
    pub distance_very_low: f32,  // 100 - 200 units
    pub distance_cull: f32,      // 200+ units - don't render
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            distance_high: 50.0,      // Increased from 20
            distance_medium: 150.0,   // Increased from 50
            distance_low: 300.0,      // Increased from 100
            distance_very_low: 500.0, // Increased from 200
            distance_cull: 1000.0,    // Increased from 300 - MUCH further culling
        }
    }
}

impl LODConfig {
    pub fn get_lod_level(&self, distance: f32) -> LODLevel {
        if distance < self.distance_high {
            LODLevel::High
        } else if distance < self.distance_medium {
            LODLevel::Medium
        } else if distance < self.distance_low {
            LODLevel::Low
        } else if distance < self.distance_very_low {
            LODLevel::VeryLow
        } else if distance < self.distance_cull {
            LODLevel::VeryLow  // Still render but very low quality
        } else {
            LODLevel::Culled   // Too far, don't render
        }
    }
}

/// Frustum for visibility culling
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [FrustumPlane; 6],  // Left, Right, Top, Bottom, Near, Far
}

#[derive(Debug, Clone, Copy)]
pub struct FrustumPlane {
    pub normal: Vec3,
    pub distance: f32,
}

impl FrustumPlane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn distance_to_point(&self, point: &Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    pub fn is_point_inside(&self, point: &Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix (row-major order)
    pub fn from_camera(camera: &Camera) -> Self {
        let vp = camera.view_projection_matrix();
        let m = vp.data;

        // Extract planes from view-projection matrix (row-major)
        let mut planes = [FrustumPlane::new(Vec3::ZERO, 0.0); 6];

        // Left plane: row3 + row0
        planes[0] = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0]),
            m[3][3] + m[3][0],
        );

        // Right plane: row3 - row0
        planes[1] = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0]),
            m[3][3] - m[3][0],
        );

        // Bottom plane: row3 + row1
        planes[2] = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1]),
            m[3][3] + m[3][1],
        );

        // Top plane: row3 - row1
        planes[3] = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1]),
            m[3][3] - m[3][1],
        );

        // Near plane: row3 + row2
        planes[4] = FrustumPlane::new(
            Vec3::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2]),
            m[3][3] + m[3][2],
        );

        // Far plane: row3 - row2
        planes[5] = FrustumPlane::new(
            Vec3::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2]),
            m[3][3] - m[3][2],
        );

        // Normalize planes
        for plane in &mut planes {
            let length = plane.normal.length();
            if length > 0.001 {
                plane.normal = plane.normal / length;
                plane.distance /= length;
            }
        }

        Self { planes }
    }

    /// Check if a sphere is visible (with radius) - VERY generous culling
    pub fn is_sphere_visible(&self, center: &Vec3, radius: f32) -> bool {
        // Use a VERY generous radius for particles
        let safe_radius = radius.max(2.0);  // At least 2 units (was 0.5)
        
        for plane in &self.planes {
            let dist = plane.distance_to_point(center);
            if dist < -safe_radius * 3.0 {  // MUCH more generous (was 1.5)
                return false;  // Only cull if REALLY outside
            }
        }
        true  // Sphere is at least partially inside all planes
    }

    /// Check if a point is visible
    pub fn is_point_visible(&self, point: &Vec3) -> bool {
        for plane in &self.planes {
            if !plane.is_point_inside(point) {
                return false;
            }
        }
        true
    }
}

/// LOD Manager - tracks all objects and their LOD states
pub struct LODManager {
    pub config: LODConfig,
    pub frustum: Frustum,
    pub stats: LODStats,
}

#[derive(Debug, Default, Clone)]
pub struct LODStats {
    pub total_objects: usize,
    pub visible_objects: usize,
    pub culled_by_distance: usize,
    pub culled_by_frustum: usize,
    pub high_lod: usize,
    pub medium_lod: usize,
    pub low_lod: usize,
    pub very_low_lod: usize,
}

impl LODManager {
    pub fn new() -> Self {
        Self {
            config: LODConfig::default(),
            frustum: Frustum {
                planes: [FrustumPlane::new(Vec3::ZERO, 0.0); 6],
            },
            stats: LODStats::default(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.frustum = Frustum::from_camera(camera);
        self.stats = LODStats::default();  // Reset stats
    }

    /// Calculate LOD level for an object
    pub fn calculate_lod(&mut self, position: &Vec3, camera_pos: &Vec3, radius: f32) -> LODLevel {
        self.stats.total_objects += 1;

        // 1. Frustum culling (most important - saves GPU work)
        if !self.frustum.is_sphere_visible(position, radius) {
            self.stats.culled_by_frustum += 1;
            return LODLevel::Culled;
        }

        // 2. Distance-based LOD
        let distance = position.distance(camera_pos);
        let lod = self.config.get_lod_level(distance);

        // Update stats
        match lod {
            LODLevel::High => self.stats.high_lod += 1,
            LODLevel::Medium => self.stats.medium_lod += 1,
            LODLevel::Low => self.stats.low_lod += 1,
            LODLevel::VeryLow => self.stats.very_low_lod += 1,
            LODLevel::Culled => {
                self.stats.culled_by_distance += 1;
                return lod;
            }
        }

        self.stats.visible_objects += 1;
        lod
    }

    /// Check if object is behind camera (occluded)
    pub fn is_behind_camera(&self, position: &Vec3, camera: &Camera) -> bool {
        let to_object = *position - camera.position;
        to_object.dot(&camera.forward) < 0.0
    }
}

/// Object metadata for LOD tracking
#[derive(Debug, Clone)]
pub struct LODObject {
    pub position: Vec3,
    pub radius: f32,
    pub current_lod: LODLevel,
    pub was_visible_last_frame: bool,
    pub frames_invisible: u32,
}

impl LODObject {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self {
            position,
            radius,
            current_lod: LODLevel::High,
            was_visible_last_frame: false,
            frames_invisible: 0,
        }
    }

    pub fn update_lod(&mut self, lod_manager: &mut LODManager, camera_pos: &Vec3) -> bool {
        let lod = lod_manager.calculate_lod(&self.position, camera_pos, self.radius);
        self.current_lod = lod;

        let is_visible = lod.should_render();
        
        if is_visible {
            self.was_visible_last_frame = true;
            self.frames_invisible = 0;
        } else {
            self.was_visible_last_frame = false;
            self.frames_invisible += 1;
        }

        is_visible
    }

    /// Should we keep simulating physics even if not visible?
    pub fn should_simulate_physics(&self) -> bool {
        // Keep simulating if recently visible or close
        self.was_visible_last_frame || self.frames_invisible < 60  // 1 second at 60fps
    }
}

