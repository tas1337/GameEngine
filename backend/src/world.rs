// Shared world state - all players see the same particles!
use std::sync::RwLock;
use uuid::Uuid;
use crate::protocol::{Vec3, Color, ParticleData, CloudData, PickableObject, WorldSnapshot};

const GRAVITY: Vec3 = Vec3 { x: 0.0, y: -80.0, z: 0.0 };  // Match client gravity for fast falling
const MAX_PARTICLES: usize = 10_000_000;  // 10 million max
const CLOUD_COUNT: usize = 20;
const CLOUD_BOUNDS: f32 = 150.0;  // Bigger cloud area for bigger ground
const GROUND_SIZE: f32 = 100.0;  // Match client ground size

pub struct World {
    /// All active particles (shared between players)
    particles: RwLock<Vec<Particle>>,
    
    /// Server-synced clouds (same positions for all players)
    clouds: RwLock<Vec<Cloud>>,
    
    /// Pickable objects (boxes that can be picked up)
    pickables: RwLock<Vec<Pickable>>,
    
    /// Time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    pub time_of_day: RwLock<f32>,
    
    /// Day/night cycle speed
    day_speed: f32,
}

#[derive(Debug, Clone)]
struct Cloud {
    position: Vec3,
    velocity: Vec3,
    scale: f32,
    is_raining: bool,  // Dark rain cloud
}

#[derive(Debug, Clone)]
struct Pickable {
    id: u32,
    position: Vec3,
    velocity: Vec3,
    held_by: Option<Uuid>,
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
        // Initialize clouds with deterministic positions (same for all players)
        let mut clouds = Vec::with_capacity(CLOUD_COUNT);
        let mut seed: u32 = 12345; // Fixed seed for deterministic clouds
        
        for _ in 0..CLOUD_COUNT {
            // Simple deterministic random
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r1 = (seed >> 16) as f32 / 65535.0;
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r2 = (seed >> 16) as f32 / 65535.0;
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r3 = (seed >> 16) as f32 / 65535.0;
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r4 = (seed >> 16) as f32 / 65535.0;
            
            let x = (r1 - 0.5) * CLOUD_BOUNDS * 2.0;
            let z = (r2 - 0.5) * CLOUD_BOUNDS * 2.0;
            let y = 150.0 + r3 * 30.0;  // Much higher in sky (150-180)
            
            // Slow drift speed, all same direction (eastward)
            let speed = 0.15 + r4 * 0.1;
            
            // Only 1 rain cloud at a time (the first one)
            let is_raining = clouds.is_empty();  // Only first cloud is rain cloud
            
            clouds.push(Cloud {
                position: Vec3 { x, y, z },
                velocity: Vec3 { 
                    x: speed,           // All drift east (+X)
                    y: 0.0, 
                    z: speed * 0.3      // Slight +Z
                },
                scale: 25.0 + r3 * 15.0,  // Much bigger flat clouds (25-40)
                is_raining,
            });
        }
        
        // Initialize pickable objects (multiple boxes spread around)
        let pickables = vec![
            Pickable {
                id: 1,
                position: Vec3 { x: 5.0, y: 1.0, z: 5.0 },
                velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                held_by: None,
            },
            Pickable {
                id: 2,
                position: Vec3 { x: -15.0, y: 1.0, z: 10.0 },
                velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                held_by: None,
            },
            Pickable {
                id: 3,
                position: Vec3 { x: 20.0, y: 1.0, z: -10.0 },
                velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                held_by: None,
            },
            Pickable {
                id: 4,
                position: Vec3 { x: -30.0, y: 1.0, z: -25.0 },
                velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                held_by: None,
            },
            Pickable {
                id: 5,
                position: Vec3 { x: 40.0, y: 1.0, z: 30.0 },
                velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
                held_by: None,
            },
        ];
        
        Self {
            particles: RwLock::new(Vec::with_capacity(MAX_PARTICLES)),
            clouds: RwLock::new(clouds),
            pickables: RwLock::new(pickables),
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
        
        // Update clouds (server-authoritative movement)
        {
            let mut clouds = self.clouds.write().unwrap();
            for cloud in clouds.iter_mut() {
                cloud.position.x += cloud.velocity.x * dt;
                cloud.position.z += cloud.velocity.z * dt;
                
                // Wrap around bounds
                if cloud.position.x > CLOUD_BOUNDS {
                    cloud.position.x = -CLOUD_BOUNDS;
                }
                if cloud.position.x < -CLOUD_BOUNDS {
                    cloud.position.x = CLOUD_BOUNDS;
                }
                if cloud.position.z > CLOUD_BOUNDS {
                    cloud.position.z = -CLOUD_BOUNDS;
                }
                if cloud.position.z < -CLOUD_BOUNDS {
                    cloud.position.z = CLOUD_BOUNDS;
                }
            }
            
            // Collect rain spawn positions from rain clouds (only 1 cloud rains at a time)
            // Only spawn 1 drop every few ticks to prevent accumulation
            static RAIN_TICK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let tick = RAIN_TICK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            let rain_positions: Vec<Vec3> = if tick % 2 == 0 {  // Spawn every 2nd tick
                clouds.iter()
                    .filter(|cloud| cloud.is_raining)
                    .filter(|cloud| cloud.position.x.abs() <= GROUND_SIZE + 30.0 && cloud.position.z.abs() <= GROUND_SIZE + 30.0)
                    .flat_map(|cloud| {
                        // Spawn 5 rain drops per tick for visible rain
                        (0..5).map(move |i| {
                            // Deterministic "random" based on cloud position and tick
                            let seed_base = (cloud.position.x * 1000.0 + cloud.position.z * 100.0 + i as f32 * 10.0 + tick as f32) as u32;
                            let r1 = ((seed_base.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as f32 / 65535.0;
                            let r2 = ((seed_base.wrapping_mul(1103515245).wrapping_add(54321)) >> 16) as f32 / 65535.0;
                            
                            let offset_x = (r1 - 0.5) * cloud.scale * 1.5;
                            let offset_z = (r2 - 0.5) * cloud.scale * 1.5;
                            
                            Vec3 {
                                x: cloud.position.x + offset_x,
                                y: cloud.position.y - 5.0,  // Start just below cloud
                                z: cloud.position.z + offset_z,
                            }
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };
            
            drop(clouds);  // Release clouds lock
            
            // Spawn rain particles (server-authoritative, all players see same rain)
            for pos in rain_positions {
                self.spawn_rain_particle(pos);
            }
        }
        
        // Update pickable objects (boxes) physics - gravity, ground collision, AND box-on-box stacking!
        {
            let mut pickables = self.pickables.write().unwrap();
            let box_half = 1.0;  // Half size of box (2x2x2 box)
            
            // First pass: apply gravity and movement
            for obj in pickables.iter_mut() {
                if obj.held_by.is_none() {
                    obj.velocity.y += GRAVITY.y * dt;
                    obj.position.x += obj.velocity.x * dt;
                    obj.position.y += obj.velocity.y * dt;
                    obj.position.z += obj.velocity.z * dt;
                }
            }
            
            // Second pass: box-on-box collision (stacking like Half-Life!)
            let box_count = pickables.len();
            for i in 0..box_count {
                if pickables[i].held_by.is_some() {
                    continue;  // Skip held boxes
                }
                
                for j in 0..box_count {
                    if i == j || pickables[j].held_by.is_some() {
                        continue;
                    }
                    
                    let pos_i = pickables[i].position;
                    let pos_j = pickables[j].position;
                    
                    // Check if boxes overlap
                    let dx = (pos_i.x - pos_j.x).abs();
                    let dy = (pos_i.y - pos_j.y).abs();
                    let dz = (pos_i.z - pos_j.z).abs();
                    
                    let overlap_x = box_half * 2.0 - dx;
                    let overlap_y = box_half * 2.0 - dy;
                    let overlap_z = box_half * 2.0 - dz;
                    
                    if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
                        // Boxes are colliding! Find smallest overlap axis
                        if overlap_y < overlap_x && overlap_y < overlap_z {
                            // Vertical collision - stack!
                            if pos_i.y > pos_j.y {
                                // Box i is on top of box j
                                pickables[i].position.y = pos_j.y + box_half * 2.0;
                                if pickables[i].velocity.y < 0.0 {
                                    pickables[i].velocity.y = 0.0;  // Stop falling
                                }
                            }
                        } else if overlap_x < overlap_z {
                            // X axis push
                            let push = if pos_i.x > pos_j.x { overlap_x * 0.5 } else { -overlap_x * 0.5 };
                            pickables[i].position.x += push;
                            pickables[i].velocity.x *= 0.5;
                        } else {
                            // Z axis push
                            let push = if pos_i.z > pos_j.z { overlap_z * 0.5 } else { -overlap_z * 0.5 };
                            pickables[i].position.z += push;
                            pickables[i].velocity.z *= 0.5;
                        }
                    }
                }
            }
            
            // Third pass: ground collision and cleanup
            for obj in pickables.iter_mut() {
                if obj.held_by.is_none() {
                    let box_bottom = obj.position.y - box_half;
                    let over_ground = obj.position.x.abs() <= GROUND_SIZE 
                                   && obj.position.z.abs() <= GROUND_SIZE;
                    
                    if over_ground && box_bottom <= 0.0 {
                        obj.position.y = box_half;
                        obj.velocity.y = -obj.velocity.y * 0.3;
                        obj.velocity.x *= 0.8;
                        obj.velocity.z *= 0.8;
                        
                        if obj.velocity.y.abs() < 0.5 { obj.velocity.y = 0.0; }
                        if obj.velocity.x.abs() < 0.1 { obj.velocity.x = 0.0; }
                        if obj.velocity.z.abs() < 0.1 { obj.velocity.z = 0.0; }
                    }
                    
                    // Fall into void - respawn
                    if obj.position.y < -50.0 {
                        obj.position = Vec3 { x: 5.0, y: 10.0, z: 5.0 };
                        obj.velocity = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        tracing::info!("📦 Box respawned after falling into void");
                    }
                }
            }
        }
        
        // Update particles
        let mut particles = self.particles.write().unwrap();
        
        // Get pickable positions for collision
        let pickables = self.pickables.read().unwrap();
        let box_positions: Vec<Vec3> = pickables.iter().map(|p| p.position).collect();
        drop(pickables);
        
        // Ground extends from -GROUND_SIZE to +GROUND_SIZE on X and Z
        // Top surface is at y=0
        
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
            
            // Check if particle is OVER the green floor
            let over_green_floor = particle.position.x.abs() <= GROUND_SIZE 
                                && particle.position.z.abs() <= GROUND_SIZE;
            
            // Check collision with boxes FIRST
            let mut hit_box = false;
            for box_pos in &box_positions {
                let box_half = 1.0; // 2x2x2 box, half size is 1
                
                if particle.position.x >= box_pos.x - box_half && particle.position.x <= box_pos.x + box_half &&
                   particle.position.y >= box_pos.y - box_half && particle.position.y <= box_pos.y + box_half &&
                   particle.position.z >= box_pos.z - box_half && particle.position.z <= box_pos.z + box_half {
                    
                    hit_box = true;
                    let dx = particle.position.x - box_pos.x;
                    let dy = particle.position.y - box_pos.y;
                    let dz = particle.position.z - box_pos.z;
                    
                    let px = box_half - dx.abs();
                    let py = box_half - dy.abs();
                    let pz = box_half - dz.abs();
                    
                    if px < py && px < pz {
                        particle.position.x = box_pos.x + box_half * if dx > 0.0 { 1.01 } else { -1.01 };
                        particle.velocity.x = -particle.velocity.x * 0.6;
                    } else if py < pz {
                        particle.position.y = box_pos.y + box_half * if dy > 0.0 { 1.01 } else { -1.01 };
                        particle.velocity.y = -particle.velocity.y * 0.6;
                    } else {
                        particle.position.z = box_pos.z + box_half * if dz > 0.0 { 1.01 } else { -1.01 };
                        particle.velocity.z = -particle.velocity.z * 0.6;
                    }
                }
            }
            
            // Ground collision - only if over green floor AND didn't hit a box
            if !hit_box && over_green_floor && particle.position.y <= 0.0 {
                particle.position.y = 0.01;  // Slightly above ground
                particle.velocity.y = -particle.velocity.y * 0.6;  // Bounce
                particle.velocity.x *= 0.92;  // Friction
                particle.velocity.z *= 0.92;
            }
            
            // Remove if fallen into the void
            if particle.position.y < -50.0 {
                return false;
            }
            
            true  // Keep particle
        });
        
        // Log stats occasionally
        if particles.len() > 0 && rand::random::<u32>() % 600 == 0 {
            tracing::info!("💫 Active particles: {}", particles.len());
        }
    }
    
    /// Spawn a single rain particle (server-authoritative)
    fn spawn_rain_particle(&self, position: Vec3) {
        let mut particles = self.particles.write().unwrap();
        
        // Limit rain particles to prevent accumulation
        let rain_count = particles.iter().filter(|p| p.owner.is_none()).count();
        if rain_count >= 2000 {
            return;  // Max 2000 rain particles at a time
        }
        
        // Check capacity
        if particles.len() >= MAX_PARTICLES {
            return;  // Skip if at capacity
        }
        
        particles.push(Particle {
            position,
            velocity: Vec3 { x: 0.0, y: -80.0, z: 0.0 },  // Very fast falling rain
            color: Color { r: 0.5, g: 0.5, b: 0.9, a: 0.8 },  // Blue-ish rain
            lifetime: 2.5,  // Shorter lifetime - rain falls fast and disappears
            age: 0.0,
            owner: None,  // Rain has no owner
        });
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
    
    /// Get cloud data for network update
    pub fn get_clouds(&self) -> Vec<CloudData> {
        let clouds = self.clouds.read().unwrap();
        clouds.iter()
            .map(|c| CloudData {
                position: c.position,
                scale: c.scale,
                is_raining: c.is_raining,
            })
            .collect()
    }
    
    /// Get pickable objects for network update
    pub fn get_pickables(&self) -> Vec<PickableObject> {
        let pickables = self.pickables.read().unwrap();
        pickables.iter()
            .map(|p| PickableObject {
                id: p.id,
                position: p.position,
                velocity: p.velocity,  // Include velocity for collision push
                held_by: p.held_by.map(|id| id.to_string()),
            })
            .collect()
    }
    
    /// Pick up an object (returns true if successful)
    pub fn pickup_object(&self, object_id: u32, player_id: Uuid) -> bool {
        let mut pickables = self.pickables.write().unwrap();
        if let Some(obj) = pickables.iter_mut().find(|p| p.id == object_id) {
            if obj.held_by.is_none() {
                obj.held_by = Some(player_id);
                tracing::info!("📦 Player {} picked up object {}", player_id, object_id);
                return true;
            }
        }
        false
    }
    
    /// Drop an object at a position with velocity
    pub fn drop_object(&self, player_id: Uuid, position: Vec3, velocity: Vec3) {
        let mut pickables = self.pickables.write().unwrap();
        if let Some(obj) = pickables.iter_mut().find(|p| p.held_by == Some(player_id)) {
            obj.held_by = None;
            obj.position = position;
            obj.velocity = velocity;
            tracing::info!("📦 Player {} dropped object {} at ({}, {}, {}) with velocity ({}, {}, {})", 
                player_id, obj.id, position.x, position.y, position.z,
                velocity.x, velocity.y, velocity.z);
        }
    }
    
    /// Update held object position (called when player moves)
    /// Instantly follows player rotation for responsive feel
    pub fn update_held_object(&self, player_id: Uuid, position: Vec3, rotation: Vec3) {
        let mut pickables = self.pickables.write().unwrap();
        if let Some(obj) = pickables.iter_mut().find(|p| p.held_by == Some(player_id)) {
            // Position object in front of player based on their look direction
            let yaw = rotation.x;
            let forward_x = -yaw.sin();
            let forward_z = -yaw.cos();
            
            // Target position: 3 units in front, at hand level
            let target_pos = Vec3 {
                x: position.x + forward_x * 3.0,
                y: position.y - 1.5,
                z: position.z + forward_z * 3.0,
            };
            
            // Fast lerp - almost instant but still smooth
            let lerp_speed = 0.8;  // Very responsive, follows rotation closely
            obj.position.x += (target_pos.x - obj.position.x) * lerp_speed;
            obj.position.y += (target_pos.y - obj.position.y) * lerp_speed;
            obj.position.z += (target_pos.z - obj.position.z) * lerp_speed;
            
            // Zero out velocity when held
            obj.velocity = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        }
    }
    
    /// Check if held box collides with players and push them (called from main)
    /// Fixed to avoid deadlock by collecting pushes first, then applying them
    pub fn push_players_from_held_boxes(&self, players: &dashmap::DashMap<Uuid, crate::player::Player>) {
        let box_half = 1.0;
        let player_radius = 1.5;
        
        // Step 1: Collect all held box positions (release lock quickly)
        let held_boxes: Vec<(Uuid, Vec3)> = {
            let pickables = self.pickables.read().unwrap();
            pickables.iter()
                .filter_map(|obj| obj.held_by.map(|holder| (holder, obj.position)))
                .collect()
        };
        
        // Step 2: Collect all push operations (don't modify while iterating)
        let mut pushes: Vec<(Uuid, f32, f32)> = Vec::new();
        
        for (holder_id, box_pos) in &held_boxes {
            for entry in players.iter() {
                let player_id = *entry.key();
                if player_id == *holder_id {
                    continue;  // Don't push the person holding it
                }
                
                let player_pos = entry.value().position;
                
                // Simple sphere-box collision
                let dx = player_pos.x - box_pos.x;
                let dz = player_pos.z - box_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();
                
                let push_dist = box_half + player_radius;
                
                // Check vertical overlap too (player height ~5 units)
                let player_bottom = player_pos.y - 5.0;
                let player_top = player_pos.y;
                let box_bottom = box_pos.y - box_half;
                let box_top = box_pos.y + box_half;
                let vertical_overlap = player_bottom < box_top && player_top > box_bottom;
                
                if vertical_overlap && dist < push_dist && dist > 0.01 {
                    let overlap = push_dist - dist;
                    let push_x = (dx / dist) * overlap * 8.0;  // MEGA push (x4)
                    let push_z = (dz / dist) * overlap * 8.0;
                    pushes.push((player_id, push_x, push_z));
                }
            }
        }
        
        // Step 3: Apply all pushes (now safe to modify)
        for (player_id, push_x, push_z) in pushes {
            if let Some(mut player) = players.get_mut(&player_id) {
                player.position.x += push_x;
                player.position.z += push_z;
            }
        }
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

