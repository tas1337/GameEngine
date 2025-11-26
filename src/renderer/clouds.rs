// Cloud system - built from scratch!
// Minecraft-style flat clouds made of clustered boxes

use crate::math::Vec3;
use crate::ecs::Color;

/// A single box within a cloud formation
pub struct CloudBox {
    pub offset: Vec3,      // Offset from cloud center
    pub scale: Vec3,       // Size of this box (flat rectangles)
}

/// A cloud formation made of multiple boxes
pub struct Cloud {
    pub position: Vec3,
    pub velocity: Vec3,
    pub boxes: Vec<CloudBox>,
    pub alpha: f32,
}

impl Cloud {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        let mut boxes = Vec::new();
        
        // Generate 5-12 boxes per cloud in a clustered formation
        let box_count = 5 + (js_sys::Math::random() * 8.0) as usize;
        
        for _ in 0..box_count {
            // Random offset from cloud center (spread horizontally)
            let offset_x = (js_sys::Math::random() as f32 - 0.5) * 30.0;
            let offset_z = (js_sys::Math::random() as f32 - 0.5) * 20.0;
            let offset_y = (js_sys::Math::random() as f32 - 0.5) * 3.0;  // Very little vertical variation
            
            // Flat rectangular boxes (wide, not tall)
            let scale_x = 8.0 + js_sys::Math::random() as f32 * 12.0;  // Wide
            let scale_z = 6.0 + js_sys::Math::random() as f32 * 10.0;  // Wide  
            let scale_y = 2.0 + js_sys::Math::random() as f32 * 2.0;   // Thin/flat
            
            boxes.push(CloudBox {
                offset: Vec3::new(offset_x, offset_y, offset_z),
                scale: Vec3::new(scale_x, scale_y, scale_z),
            });
        }
        
        Self {
            position,
            velocity,
            boxes,
            alpha: 0.9,
        }
    }

    pub fn update(&mut self, dt: f32, bounds: f32) {
        self.position += self.velocity * dt;
        
        // Wrap around when clouds go too far
        if self.position.x > bounds {
            self.position.x = -bounds;
        }
        if self.position.x < -bounds {
            self.position.x = bounds;
        }
        if self.position.z > bounds {
            self.position.z = -bounds;
        }
        if self.position.z < -bounds {
            self.position.z = bounds;
        }
    }

    pub fn get_color(&self, is_night: bool) -> Color {
        if is_night {
            // Dark gray blocky clouds at night (Minecraft style)
            Color::new(0.3, 0.3, 0.4, self.alpha)
        } else {
            // Pure white blocky clouds during day (Minecraft style)
            Color::new(0.95, 0.95, 0.95, self.alpha)
        }
    }
    
    /// Get all box instances for this cloud
    pub fn get_box_instances(&self, is_night: bool) -> Vec<(Vec3, Vec3, Color)> {
        let color = self.get_color(is_night);
        self.boxes.iter().map(|b| {
            (
                self.position + b.offset,
                b.scale,
                color,
            )
        }).collect()
    }
}

pub struct CloudSystem {
    pub clouds: Vec<Cloud>,
    pub bounds: f32,
}

impl CloudSystem {
    pub fn new(count: usize, bounds: f32) -> Self {
        let mut clouds = Vec::with_capacity(count);
        
        for _ in 0..count {
            let x = (js_sys::Math::random() as f32 - 0.5) * bounds * 2.0;
            let z = (js_sys::Math::random() as f32 - 0.5) * bounds * 2.0;
            let y = 150.0 + js_sys::Math::random() as f32 * 30.0; // Very high in sky
            
            let speed = 0.15 + js_sys::Math::random() as f32 * 0.1;  // Slow drift
            
            // All clouds move in the same direction (gentle eastward drift)
            clouds.push(Cloud::new(
                Vec3::new(x, y, z),
                Vec3::new(speed, 0.0, speed * 0.3),  // Mostly +X with slight +Z
            ));
        }
        
        Self { clouds, bounds }
    }

    pub fn update(&mut self, dt: f32) {
        for cloud in &mut self.clouds {
            cloud.update(dt, self.bounds);
        }
    }

    /// Get ALL box instances from ALL clouds (fully instanced rendering)
    pub fn get_instances(&self, is_night: bool) -> Vec<(Vec3, Vec3, Color)> {
        self.clouds.iter()
            .flat_map(|cloud| cloud.get_box_instances(is_night))
            .collect()
    }
}
