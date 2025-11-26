// Network protocol - efficient binary serialization
// MUST MATCH CLIENT (src/network/mod.rs)!
use serde::{Serialize, Deserialize};

/// Vector3 (matches client-side Vec3)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Color (matches client-side Color) - WITH ALPHA!
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Cloud data (server-synced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudData {
    pub position: Vec3,
    pub scale: f32,
    pub is_raining: bool,  // Dark rain cloud - clients spawn rain locally
}

/// Pickable object data (server-synced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickableObject {
    pub id: u32,
    pub position: Vec3,
    pub velocity: Vec3,  // Box velocity for collision push
    pub held_by: Option<String>,  // Player ID holding it, None if on ground
}

/// Messages from client to server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Update player position/rotation
    PlayerUpdate {
        position: Vec3,
        rotation: Vec3,
        velocity: Vec3,
    },
    
    /// Spawn particles (server-authoritative)
    SpawnParticles {
        positions: Vec<Vec3>,
        velocities: Vec<Vec3>,
        colors: Vec<Color>,
    },
    
    /// Pick up or drop an object
    PickupObject {
        object_id: u32,
    },
    
    /// Drop the currently held object
    DropObject {
        position: Vec3,
        velocity: Vec3,
    },
    
    /// Report a kill (knocked someone off)
    ReportKill {
        victim_id: String,
    },
    
    /// Report own death (fell off)
    ReportDeath,
    
    /// Ping (latency check)
    Ping,
}

/// Remote player data sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePlayer {
    pub id: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
    pub score: u32,  // King of the hill score
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub id: String,
    pub score: u32,
    pub rank: u32,
}

/// Messages from server to client
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Welcome message with player ID
    Welcome {
        player_id: String,
        world_state: WorldSnapshot,
    },
    
    /// World update (sent every tick)
    WorldUpdate {
        players: Vec<RemotePlayer>,
        particles: Vec<ParticleData>,
        clouds: Vec<CloudData>,
        pickables: Vec<PickableObject>,
        sun_time: f32,
        leaderboard: Vec<LeaderboardEntry>,  // Top 5 + current player if not in top 5
    },
    
    /// Pong response
    Pong {
        timestamp: u64,
    },
}

/// Snapshot of world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub time_of_day: f32,
    pub particle_count: usize,
    pub player_count: usize,
}

/// Particle data (compressed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleData {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Color,
    pub lifetime: f32,
}

// Basic math helpers for Vec3 (minimal set for server physics)
impl std::ops::Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let inv = if rhs.abs() > 0.00001 { 1.0 / rhs } else { 0.0 };
        Self {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
        }
    }
}

