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
        sun_time: f32,
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

