// Cloud system - built from scratch!

use crate::math::Vec3;
use crate::ecs::Color;

pub struct Cloud {
    pub position: Vec3,
    pub velocity: Vec3,
    pub scale: f32,
    pub alpha: f32,
}

impl Cloud {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self {
            position,
            velocity,
            scale: 8.0 + js_sys::Math::random() as f32 * 5.0,  // Bigger, blockier
            alpha: 0.9,  // More opaque for Minecraft look
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
            let y = 20.0 + js_sys::Math::random() as f32 * 30.0; // Float high in sky
            
            let speed = 0.5 + js_sys::Math::random() as f32 * 1.5;
            let angle = js_sys::Math::random() as f32 * std::f32::consts::PI * 2.0;
            
            clouds.push(Cloud::new(
                Vec3::new(x, y, z),
                Vec3::new(angle.cos() * speed, 0.0, angle.sin() * speed),
            ));
        }
        
        Self { clouds, bounds }
    }

    pub fn update(&mut self, dt: f32) {
        for cloud in &mut self.clouds {
            cloud.update(dt, self.bounds);
        }
    }

    pub fn get_instances(&self, is_night: bool) -> Vec<(Vec3, Vec3, Color)> {
        self.clouds.iter().map(|cloud| {
            (
                cloud.position,
                Vec3::splat(cloud.scale),
                cloud.get_color(is_night),
            )
        }).collect()
    }
}

