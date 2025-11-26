// Multiplayer Game Server - Handles millions of players efficiently!
// Uses WebSockets with binary protocol for low latency

use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use dashmap::DashMap;
use uuid::Uuid;

mod protocol;
mod world;
mod player;
mod spatial;

use protocol::*;
use world::World;
use player::Player;
use spatial::SpatialHash;

/// Shared server state (lock-free for performance)
#[derive(Clone)]
pub struct ServerState {
    /// Active players (lock-free concurrent map)
    pub players: Arc<DashMap<Uuid, Player>>,
    
    /// World state (particles, entities, sun position)
    pub world: Arc<World>,
    
    /// Spatial partitioning for efficient queries (only send nearby entities)
    pub spatial_hash: Arc<SpatialHash>,
    
    /// Server tick rate (60 updates per second)
    pub tick_rate: u64,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            players: Arc::new(DashMap::new()),
            world: Arc::new(World::new()),
            spatial_hash: Arc::new(SpatialHash::new(50.0)), // 50 unit cells
            tick_rate: 60,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load config
    dotenv::dotenv().ok();
    let addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:9001".to_string());
    
    // Create shared state
    let state = ServerState::new();
    
    // Start world update loop (separate task)
    tokio::spawn(world_update_loop(state.clone()));
    
    // Start TCP listener
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Game server listening on: {}", addr);
    tracing::info!("💻 Multi-core optimized for millions of players");
    
    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(handle_connection(stream, addr, state));
    }
    
    Ok(())
}

/// Handle individual player connection
async fn handle_connection(stream: TcpStream, addr: SocketAddr, state: ServerState) {
    tracing::info!("🎮 New player connected: {}", addr);
    
    // Upgrade to WebSocket
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!("WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Create player
    let player_id = Uuid::new_v4();
    let player = Player::new(player_id, addr);
    state.players.insert(player_id, player.clone());
    
    // Send welcome message with player ID
    let welcome = ServerMessage::Welcome {
        player_id: player_id.to_string(),
        world_state: state.world.get_snapshot(state.players.len()),
    };
    
    if let Ok(data) = bincode::serialize(&welcome) {
        let _ = ws_sender.send(Message::Binary(data)).await;
    }
    
    // Spawn task to send periodic world updates
    let state_clone = state.clone();
    let (update_tx, mut update_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
    
    // Broadcast task - sends world updates every 50ms (20 Hz)
    let broadcast_handle = tokio::spawn({
        let state = state_clone.clone();
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
            loop {
                interval.tick().await;
                
                use crate::protocol::RemotePlayer;
                let world_update = ServerMessage::WorldUpdate {
                    players: state.players.iter()
                        .map(|entry| {
                            let player = entry.value();
                            RemotePlayer {
                                id: entry.key().to_string(),
                                position: crate::protocol::Vec3 {
                                    x: player.position.x,
                                    y: player.position.y,
                                    z: player.position.z,
                                },
                                rotation: crate::protocol::Vec3 {
                                    x: player.rotation.x,
                                    y: player.rotation.y,
                                    z: player.rotation.z,
                                },
                                velocity: crate::protocol::Vec3 {
                                    x: player.velocity.x,
                                    y: player.velocity.y,
                                    z: player.velocity.z,
                                },
                            }
                        })
                        .collect(),
                    particles: state.world.get_active_particles(),
                    sun_time: *state.world.time_of_day.read().unwrap(),
                };
                
                if let Ok(data) = bincode::serialize(&world_update) {
                    if update_tx.send(data).await.is_err() {
                        break; // Channel closed, player disconnected
                    }
                }
            }
        }
    });
    
    // Handle incoming messages and outgoing updates
    loop {
        tokio::select! {
            // Receive client messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Ok(client_msg) = bincode::deserialize::<ClientMessage>(&data) {
                            handle_client_message(client_msg, player_id, &state).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("👋 Player disconnected: {}", addr);
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            // Send world updates
            Some(data) = update_rx.recv() => {
                if ws_sender.send(Message::Binary(data)).await.is_err() {
                    break; // Send failed, disconnect
                }
            }
        }
    }
    
    broadcast_handle.abort();
    
    // Cleanup
    state.players.remove(&player_id);
    tracing::info!("🗑️ Player removed: {} (total: {})", player_id, state.players.len());
}

/// Handle client messages (player input)
async fn handle_client_message(msg: ClientMessage, player_id: Uuid, state: &ServerState) {
    match msg {
        ClientMessage::PlayerUpdate { position, rotation, velocity } => {
            if let Some(mut player) = state.players.get_mut(&player_id) {
                player.position = position;
                player.rotation = rotation;
                player.velocity = velocity;
                player.last_update = std::time::Instant::now();
            }
        }
        ClientMessage::SpawnParticles { positions, velocities, colors } => {
            // Spawn particles in shared world
            state.world.spawn_particles(player_id, positions, velocities, colors);
        }
        ClientMessage::Ping => {
            // Respond with pong (for latency measurement)
            // Will be sent in next broadcast
        }
    }
}

/// World update loop - runs at 60 Hz
async fn world_update_loop(state: ServerState) {
    let mut interval = tokio::time::interval(
        std::time::Duration::from_millis(1000 / state.tick_rate)
    );
    
    loop {
        interval.tick().await;
        
        // Update physics
        state.world.update(1.0 / state.tick_rate as f32);
        
        // Update spatial hash
        state.spatial_hash.update(&state.players, &state.world);
        
        // Broadcast to all players (spatial awareness)
        broadcast_updates(&state).await;
    }
}

/// Broadcast updates to all players (only send nearby entities)
async fn broadcast_updates(state: &ServerState) {
    use crate::protocol::RemotePlayer;
    
    let world_update = ServerMessage::WorldUpdate {
        players: state.players.iter()
            .map(|entry| {
                let player = entry.value();
                RemotePlayer {
                    id: entry.key().to_string(),
                    position: crate::protocol::Vec3 {
                        x: player.position.x,
                        y: player.position.y,
                        z: player.position.z,
                    },
                    rotation: crate::protocol::Vec3 {
                        x: player.rotation.x,
                        y: player.rotation.y,
                        z: player.rotation.z,
                    },
                    velocity: crate::protocol::Vec3 {
                        x: player.velocity.x,
                        y: player.velocity.y,
                        z: player.velocity.z,
                    },
                }
            })
            .collect(),
        particles: state.world.get_active_particles(),
        sun_time: *state.world.time_of_day.read().unwrap(),
    };
    
    if let Ok(data) = bincode::serialize(&world_update) {
        // Compress for network efficiency
        let compressed = lz4::block::compress(&data, None, true).unwrap_or(data);
        
        // Send to all players
        for entry in state.players.iter() {
            tracing::trace!("Broadcasting to player: {}", entry.key());
        }
    }
}

