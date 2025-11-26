// Multiplayer networking - WebSocket client
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;
use crate::math::Vec3;
use crate::ecs::Color;

/// Cloud data (server-synced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudData {
    pub position: Vec3,
    pub scale: f32,
    pub is_raining: bool,  // Client spawns rain locally for rain clouds
}

/// Pickable object data (server-synced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickableObject {
    pub id: u32,
    pub position: Vec3,
    pub velocity: Vec3,  // Box velocity for collision push
    pub held_by: Option<String>,  // Player ID holding it, None if on ground
}

/// Network message types (matches backend protocol)
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    PlayerUpdate {
        position: Vec3,
        rotation: Vec3,
        velocity: Vec3,
    },
    SpawnParticles {
        positions: Vec<Vec3>,
        velocities: Vec<Vec3>,
        colors: Vec<Color>,
    },
    PickupObject {
        object_id: u32,
    },
    DropObject {
        position: Vec3,
        velocity: Vec3,
    },
    ReportKill {
        victim_id: String,
    },
    ReportDeath,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome {
        player_id: String,
        world_state: WorldSnapshot,
    },
    WorldUpdate {
        players: Vec<RemotePlayer>,
        particles: Vec<ParticleData>,
        clouds: Vec<CloudData>,
        pickables: Vec<PickableObject>,
        sun_time: f32,
        leaderboard: Vec<LeaderboardEntry>,
    },
    Pong {
        timestamp: u64,
    },
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub id: String,
    pub score: u32,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub time_of_day: f32,
    pub particle_count: usize,
    pub player_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePlayer {
    pub id: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
    pub score: u32,  // King of the hill score
}

/// Interpolated remote player (for smooth rendering)
#[derive(Debug, Clone)]
pub struct InterpolatedPlayer {
    pub id: String,
    pub position: Vec3,          // Current interpolated position
    pub target_position: Vec3,   // Target from server
    pub rotation: Vec3,
    pub target_rotation: Vec3,
    pub velocity: Vec3,
    pub score: u32,  // King of the hill score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleData {
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Color,
    pub lifetime: f32,
}

/// Network manager - handles multiplayer connections
pub struct NetworkManager {
    ws: Option<WebSocket>,
    pub connected: bool,
    pub player_id: Option<String>,
    pub remote_players: Vec<InterpolatedPlayer>,  // Interpolated for smooth rendering
    pub server_time: f32,
    pub target_server_time: f32,  // For smooth time interpolation
    pub player_count: usize,
    pub server_clouds: Vec<CloudData>,
    pub target_clouds: Vec<CloudData>,  // For smooth cloud movement
    pub pickables: Vec<PickableObject>,
    pub target_pickables: Vec<PickableObject>,  // For smooth box movement
    pub server_particles: Vec<ParticleData>,  // Server-synced particles (rain, etc.)
    pub leaderboard: Vec<LeaderboardEntry>,  // Top 5 players + current if not in top 5
    pub my_score: u32,  // Current player's score
    pub my_rank: u32,   // Current player's rank
    // Shared state for callbacks
    shared_state: Rc<RefCell<SharedNetworkState>>,
}

#[derive(Clone)]
struct SharedNetworkState {
    player_id: Option<String>,
    remote_players: Vec<RemotePlayer>,
    server_time: f32,
    player_count: usize,
    server_clouds: Vec<CloudData>,
    pickables: Vec<PickableObject>,
    server_particles: Vec<ParticleData>,  // Server-synced particles
    leaderboard: Vec<LeaderboardEntry>,
}

impl NetworkManager {
    pub fn new() -> Self {
        let shared_state = Rc::new(RefCell::new(SharedNetworkState {
            player_id: None,
            remote_players: Vec::new(),
            server_time: 0.0,
            player_count: 0,
            server_clouds: Vec::new(),
            pickables: Vec::new(),
            server_particles: Vec::new(),
            leaderboard: Vec::new(),
        }));
        
        Self {
            ws: None,
            connected: false,
            player_id: None,
            remote_players: Vec::new(),
            server_time: 0.0,
            target_server_time: 0.0,
            player_count: 0,
            server_clouds: Vec::new(),
            target_clouds: Vec::new(),
            pickables: Vec::new(),
            target_pickables: Vec::new(),
            server_particles: Vec::new(),
            leaderboard: Vec::new(),
            my_score: 0,
            my_rank: 0,
            shared_state,
        }
    }
    
    /// Sync shared state back to main state with INTERPOLATION
    pub fn sync_state(&mut self) {
        if let Ok(state) = self.shared_state.try_borrow() {
            self.player_id = state.player_id.clone();
            self.player_count = state.player_count;
            
            // Set TARGET values (we'll interpolate towards these)
            self.target_server_time = state.server_time;
            self.target_clouds = state.server_clouds.clone();
            self.target_pickables = state.pickables.clone();
            
            // Update/add remote players with interpolation targets
            for server_player in &state.remote_players {
                if let Some(existing) = self.remote_players.iter_mut().find(|p| p.id == server_player.id) {
                    // Update existing player's target
                    existing.target_position = server_player.position;
                    existing.target_rotation = server_player.rotation;
                    existing.velocity = server_player.velocity;
                    existing.score = server_player.score;
                } else {
                    // New player - start at target position
                    self.remote_players.push(InterpolatedPlayer {
                        id: server_player.id.clone(),
                        position: server_player.position,
                        target_position: server_player.position,
                        rotation: server_player.rotation,
                        target_rotation: server_player.rotation,
                        velocity: server_player.velocity,
                        score: server_player.score,
                    });
                }
            }
            
            // Remove disconnected players
            let server_ids: Vec<_> = state.remote_players.iter().map(|p| &p.id).collect();
            self.remote_players.retain(|p| server_ids.contains(&&p.id));
            
            // Update leaderboard
            self.leaderboard = state.leaderboard.clone();
            
            // Sync server particles (rain, etc.) - no interpolation needed, just replace
            self.server_particles = state.server_particles.clone();
            
            // Find our score and rank
            if let Some(ref my_id) = self.player_id {
                // First check leaderboard
                if let Some(entry) = self.leaderboard.iter().find(|e| &e.id == my_id) {
                    self.my_score = entry.score;
                    self.my_rank = entry.rank;
                } else {
                    // Not in top 5, find our score from player list
                    if let Some(me) = state.remote_players.iter().find(|p| &p.id == my_id) {
                        self.my_score = me.score;
                        // Calculate rank (count players with higher score + 1)
                        self.my_rank = state.remote_players.iter().filter(|p| p.score > me.score).count() as u32 + 1;
                    }
                }
            }
        }
    }
    
    /// Interpolate all values towards targets (call every frame)
    pub fn interpolate(&mut self, lerp_factor: f32) {
        // Smooth time interpolation
        self.server_time = self.server_time + (self.target_server_time - self.server_time) * lerp_factor;
        
        // Smooth player interpolation
        for player in &mut self.remote_players {
            player.position.x += (player.target_position.x - player.position.x) * lerp_factor;
            player.position.y += (player.target_position.y - player.position.y) * lerp_factor;
            player.position.z += (player.target_position.z - player.position.z) * lerp_factor;
            
            player.rotation.x += (player.target_rotation.x - player.rotation.x) * lerp_factor;
            player.rotation.y += (player.target_rotation.y - player.rotation.y) * lerp_factor;
        }
        
        // Smooth cloud interpolation
        for (cloud, target) in self.server_clouds.iter_mut().zip(self.target_clouds.iter()) {
            cloud.position.x += (target.position.x - cloud.position.x) * lerp_factor;
            cloud.position.y += (target.position.y - cloud.position.y) * lerp_factor;
            cloud.position.z += (target.position.z - cloud.position.z) * lerp_factor;
        }
        // Add new clouds if target has more
        if self.target_clouds.len() > self.server_clouds.len() {
            for i in self.server_clouds.len()..self.target_clouds.len() {
                self.server_clouds.push(self.target_clouds[i].clone());
            }
        }
        
        // Smooth pickable (box) interpolation
        for (pickable, target) in self.pickables.iter_mut().zip(self.target_pickables.iter()) {
            pickable.position.x += (target.position.x - pickable.position.x) * lerp_factor;
            pickable.position.y += (target.position.y - pickable.position.y) * lerp_factor;
            pickable.position.z += (target.position.z - pickable.position.z) * lerp_factor;
            pickable.held_by = target.held_by.clone();
        }
        // Add new pickables if target has more
        if self.target_pickables.len() > self.pickables.len() {
            for i in self.pickables.len()..self.target_pickables.len() {
                self.pickables.push(self.target_pickables[i].clone());
            }
        }
    }
    
    /// Connect to multiplayer server
    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("🌐 Connecting to: {}", url).into());
        
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // Setup event handlers
        let _ws_clone = ws.clone();  // Kept for potential future use
        let onopen = Closure::wrap(Box::new(move |_| {
            web_sys::console::log_1(&"✅ Connected to multiplayer server!".into());
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        
        let shared_state = self.shared_state.clone();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&abuf);
                let mut bytes = vec![0; array.length() as usize];
                array.copy_to(&mut bytes);
                
                // Deserialize message
                if let Ok(msg) = bincode::deserialize::<ServerMessage>(&bytes) {
                    if let Ok(mut state) = shared_state.try_borrow_mut() {
                        match msg {
                            ServerMessage::Welcome { player_id, world_state } => {
                                state.player_id = Some(player_id.clone());
                                state.server_time = world_state.time_of_day;
                                state.player_count = world_state.player_count;
                                web_sys::console::log_1(&format!("🎮 Player ID: {} | Players: {}", player_id, world_state.player_count).into());
                            }
                            ServerMessage::WorldUpdate { players, particles, clouds, pickables, sun_time, leaderboard } => {
                                state.remote_players = players.clone();
                                state.server_time = sun_time;
                                state.player_count = players.len();
                                state.server_clouds = clouds;
                                state.pickables = pickables;
                                state.server_particles = particles;  // Store server particles!
                                state.leaderboard = leaderboard;
                            }
                            ServerMessage::Pong { timestamp: _ } => {
                                // Ping response
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            web_sys::console::error_1(&format!("❌ WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
        
        let onclose = Closure::wrap(Box::new(move |_: CloseEvent| {
            web_sys::console::log_1(&"👋 Disconnected from server".into());
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
        
        self.ws = Some(ws);
        self.connected = true;
        
        Ok(())
    }
    
    /// Send player update to server
    pub fn send_player_update(&self, position: Vec3, rotation: Vec3, velocity: Vec3) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::PlayerUpdate {
                    position,
                    rotation,
                    velocity,
                };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Send particle spawn to server
    pub fn send_spawn_particles(&self, positions: Vec<Vec3>, velocities: Vec<Vec3>, colors: Vec<Color>) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::SpawnParticles {
                    positions,
                    velocities,
                    colors,
                };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Send pickup object request to server
    pub fn send_pickup_object(&self, object_id: u32) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::PickupObject { object_id };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Send drop object to server (legacy - no velocity)
    pub fn send_drop_object(&self, position: Vec3) -> Result<(), JsValue> {
        self.send_drop_object_with_velocity(position, Vec3::ZERO)
    }
    
    /// Send drop object to server with velocity (for throwing)
    pub fn send_drop_object_with_velocity(&self, position: Vec3, velocity: Vec3) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::DropObject { position, velocity };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Report a kill (knocked someone off the platform)
    pub fn send_report_kill(&self, victim_id: String) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::ReportKill { victim_id };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Report own death (fell off the platform)
    pub fn send_report_death(&self) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = ClientMessage::ReportDeath;
                
                if let Ok(data) = bincode::serialize(&msg) {
                    ws.send_with_u8_array(&data)?;
                }
            }
        }
        Ok(())
    }
    
    /// Check if local player is holding an object
    pub fn is_holding_object(&self) -> bool {
        if let Some(ref my_id) = self.player_id {
            self.pickables.iter().any(|p| p.held_by.as_ref() == Some(my_id))
        } else {
            false
        }
    }
    
    /// Get the nearest pickable object within range
    pub fn get_nearest_pickable(&self, position: Vec3, range: f32) -> Option<u32> {
        let mut nearest: Option<(u32, f32)> = None;
        
        for obj in &self.pickables {
            // Skip if already held
            if obj.held_by.is_some() {
                continue;
            }
            
            let dx = obj.position.x - position.x;
            let dy = obj.position.y - position.y;
            let dz = obj.position.z - position.z;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            
            if dist <= range {
                if nearest.is_none() || dist < nearest.unwrap().1 {
                    nearest = Some((obj.id, dist));
                }
            }
        }
        
        nearest.map(|(id, _)| id)
    }
    
    /// Update from server message (converts RemotePlayer to InterpolatedPlayer)
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome { player_id, world_state } => {
                self.player_id = Some(player_id.clone());
                self.target_server_time = world_state.time_of_day;
                self.server_time = world_state.time_of_day;
                self.player_count = world_state.player_count;
                web_sys::console::log_1(&format!("🎮 Player ID: {}", player_id).into());
            }
            ServerMessage::WorldUpdate { players, particles, clouds, pickables, sun_time, leaderboard } => {
                // Update targets for interpolation
                self.target_server_time = sun_time;
                self.target_clouds = clouds;
                self.target_pickables = pickables;
                self.server_particles = particles;  // Store server particles!
                self.player_count = players.len() + 1;
                self.leaderboard = leaderboard;
                
                // Update remote players with interpolation
                for server_player in &players {
                    if let Some(existing) = self.remote_players.iter_mut().find(|p| p.id == server_player.id) {
                        existing.target_position = server_player.position;
                        existing.target_rotation = server_player.rotation;
                        existing.velocity = server_player.velocity;
                        existing.score = server_player.score;
                    } else {
                        self.remote_players.push(InterpolatedPlayer {
                            id: server_player.id.clone(),
                            position: server_player.position,
                            target_position: server_player.position,
                            rotation: server_player.rotation,
                            target_rotation: server_player.rotation,
                            velocity: server_player.velocity,
                            score: server_player.score,
                        });
                    }
                }
                
                // Remove disconnected players
                let server_ids: Vec<_> = players.iter().map(|p| &p.id).collect();
                self.remote_players.retain(|p| server_ids.contains(&&p.id));
            }
            ServerMessage::Pong { timestamp } => {
                let now = js_sys::Date::now() as u64;
                let ping = now - timestamp;
                web_sys::console::log_1(&format!("📶 Ping: {}ms", ping).into());
            }
        }
    }
}

