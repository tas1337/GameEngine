// Player state
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::net::SocketAddr;
use std::time::Instant;
use crate::protocol::Vec3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
    
    #[serde(skip, default = "default_instant")]
    pub last_update: Instant,
    
    #[serde(skip, default = "default_addr")]
    pub addr: SocketAddr,
}

fn default_instant() -> Instant {
    Instant::now()
}

fn default_addr() -> SocketAddr {
    "0.0.0.0:0".parse().unwrap()
}

impl Player {
    pub fn new(id: Uuid, addr: SocketAddr) -> Self {
        Self {
            id,
            position: Vec3 { x: 0.0, y: 2.0, z: 5.0 },  // Default spawn
            rotation: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            last_update: Instant::now(),
            addr,
        }
    }
}

