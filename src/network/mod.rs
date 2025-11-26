// Multiplayer networking - WebSocket client
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;
use crate::math::Vec3;
use crate::ecs::Color;

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
        sun_time: f32,
    },
    Pong {
        timestamp: u64,
    },
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
    pub remote_players: Vec<RemotePlayer>,
    pub server_time: f32,
    pub player_count: usize,
    // Shared state for callbacks
    shared_state: Rc<RefCell<SharedNetworkState>>,
}

#[derive(Clone)]
struct SharedNetworkState {
    player_id: Option<String>,
    remote_players: Vec<RemotePlayer>,
    server_time: f32,
    player_count: usize,
}

impl NetworkManager {
    pub fn new() -> Self {
        let shared_state = Rc::new(RefCell::new(SharedNetworkState {
            player_id: None,
            remote_players: Vec::new(),
            server_time: 0.0,
            player_count: 0,
        }));
        
        Self {
            ws: None,
            connected: false,
            player_id: None,
            remote_players: Vec::new(),
            server_time: 0.0,
            player_count: 0,
            shared_state,
        }
    }
    
    /// Sync shared state back to main state
    pub fn sync_state(&mut self) {
        if let Ok(state) = self.shared_state.try_borrow() {
            self.player_id = state.player_id.clone();
            self.remote_players = state.remote_players.clone();
            self.server_time = state.server_time;
            self.player_count = state.player_count;
        }
    }
    
    /// Connect to multiplayer server
    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("🌐 Connecting to: {}", url).into());
        
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // Setup event handlers
        let ws_clone = ws.clone();
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
                            ServerMessage::WorldUpdate { players, particles: _, sun_time } => {
                                state.remote_players = players.clone();
                                state.server_time = sun_time;
                                state.player_count = players.len();
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
    
    /// Update from server message
    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome { player_id, world_state } => {
                self.player_id = Some(player_id.clone());
                self.server_time = world_state.time_of_day;
                self.player_count = world_state.player_count;
                web_sys::console::log_1(&format!("🎮 Player ID: {}", player_id).into());
            }
            ServerMessage::WorldUpdate { players, particles, sun_time } => {
                self.remote_players = players;
                self.server_time = sun_time;
                self.player_count = self.remote_players.len() + 1;
            }
            ServerMessage::Pong { timestamp } => {
                // Calculate ping
                let now = js_sys::Date::now() as u64;
                let ping = now - timestamp;
                web_sys::console::log_1(&format!("📶 Ping: {}ms", ping).into());
            }
        }
    }
}

