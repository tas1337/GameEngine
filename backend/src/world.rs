// Shared world state - all players see the same particles!
use std::sync::RwLock;
use uuid::Uuid;
use crate::protocol::{Vec3, Color, ParticleData, WorldSnapshot};

const GRAVITY: Vec3 = Vec3 { x: 0.0, y: -9.8, z: 0.0 };
const MAX_PARTICLES: usize = 10_000_000;  // 10 million max

pub struct World {
    /// All active particles (shared between players)
    particles: RwLock<Vec<Particle>>,
    
    /// Time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    pub time_of_day: RwLock<f32>,
    
    /// Day/night cycle speed
    day_speed: f32,
}

#[derive(Debug, Clone)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    color: Color,
    lifetime: f32,
    age: f32,
    owner: Option<Uuid>,  // Which player spawned it
}

impl World {
    pub fn new() -> Self {
        Self {
            particles: RwLock::new(Vec::with_capacity(MAX_PARTICLES)),
            time_of_day: RwLock::new(0.25),  // Start at sunrise
            day_speed: 0.0001,  // Slow day/night cycle
        }
    }
    
    /// Update physics (called every tick)
    pub fn update(&self, dt: f32) {
        // Update time of day
        {
            let mut time = self.time_of_day.write().unwrap();
            *time = (*time + self.day_speed) % 1.0;
        }
        
        // Update particles
        let mut particles = self.particles.write().unwrap();
        
        // Apply physics to all particles
        particles.retain_mut(|particle| {
            // Age
            particle.age += dt;
            if particle.age > particle.lifetime {
                return false;  // Remove dead particles
            }
            
            // Gravity
            particle.velocity.x += GRAVITY.x * dt;
            particle.velocity.y += GRAVITY.y * dt;
            particle.velocity.z += GRAVITY.z * dt;
            
            // Update position
            particle.position.x += particle.velocity.x * dt;
            particle.position.y += particle.velocity.y * dt;
            particle.position.z += particle.velocity.z * dt;
            
            // Ground collision
            if particle.position.y < 0.0 {
                particle.position.y = 0.0;
                particle.velocity.y = -particle.velocity.y * 0.6;  // Bounce
                particle.velocity.x *= 0.95;  // Friction
                particle.velocity.z *= 0.95;
            }
            
            true  // Keep particle
        });
        
        // Log stats occasionally
        if particles.len() > 0 && rand::random::<u32>() % 600 == 0 {
            tracing::info!("💫 Active particles: {}", particles.len());
        }
    }
    
    /// Spawn particles (server-authoritative)
    pub fn spawn_particles(
        &self,
        owner: Uuid,
        positions: Vec<Vec3>,
        velocities: Vec<Vec3>,
        colors: Vec<Color>,
    ) {
        let mut particles = self.particles.write().unwrap();
        
        // Check capacity
        if particles.len() + positions.len() > MAX_PARTICLES {
            tracing::warn!("⚠️ Max particles reached, ignoring spawn request");
            return;
        }
        
        // Spawn all particles
        for i in 0..positions.len() {
            particles.push(Particle {
                position: positions[i],
                velocity: velocities[i],
                color: colors[i],
                lifetime: 5.0,  // 5 second lifetime
                age: 0.0,
                owner: Some(owner),
            });
        }
        
        tracing::debug!("✨ Spawned {} particles (total: {})", positions.len(), particles.len());
    }
    
    /// Get snapshot of world state
    pub fn get_snapshot(&self, player_count: usize) -> WorldSnapshot {
        let particles = self.particles.read().unwrap();
        let time = self.time_of_day.read().unwrap();
        
        WorldSnapshot {
            time_of_day: *time,
            particle_count: particles.len(),
            player_count,
        }
    }
    
    /// Get active particles for network update
    pub fn get_active_particles(&self) -> Vec<ParticleData> {
        let particles = self.particles.read().unwrap();
        
        // Only send particles that are moving (optimization)
        particles.iter()
            .filter(|p| p.velocity.x.abs() > 0.1 || p.velocity.y.abs() > 0.1 || p.velocity.z.abs() > 0.1)
            .map(|p| ParticleData {
                position: p.position,
                velocity: p.velocity,
                color: p.color,
                lifetime: p.lifetime - p.age,
            })
            .collect()
    }
}

// Random number generation (simple)
mod rand {
    use std::cell::Cell;
    
    thread_local! {
        static SEED: Cell<u32> = Cell::new(0x12345678);
    }
    
    pub fn random<T: From<u32>>() -> T {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            seed.set(s);
            T::from(s)
        })
    }
}

