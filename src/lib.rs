// Game Engine - 100% from scratch with WebGL!
// No frameworks, just raw WebGL and Rust (like Three.js but in Rust)

pub mod math;
pub mod core;
pub mod renderer;
pub mod ecs;
pub mod input;
pub mod assets;
pub mod scene;
pub mod physics;
pub mod worker;
pub mod network;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::WebGl2RenderingContext as GL;

// JS interop for UI prompts
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn showPickupPrompt(show: bool);
    
    #[wasm_bindgen(js_namespace = window)]
    fn showHeldItem(show: bool);
}

use math::*;
use core::*;
use renderer::*;
use ecs::*;
use input::*;
use assets::*;
use scene::*;
use physics::*;
use worker::*;
use network::*;

/// Main game state
pub struct GameState {
    engine: Engine,
    gl_context: WebGLContext,
    camera: Camera,
    scene: Scene,
    input: Input,
    
    // Rendering resources
    shader_program: ShaderProgram,
    particle_vbo: GLBuffer,
    particle_ibo: GLBuffer,
    particle_index_count: i32,
    
    // Ground mesh
    ground_vbo: GLBuffer,
    ground_ibo: GLBuffer,
    ground_index_count: i32,
    ground_plane: Plane,
    
    // Player mesh (pill/capsule)
    player_vbo: GLBuffer,
    player_ibo: GLBuffer,
    player_index_count: i32,
    
    // Sun mesh
    sun_vbo: GLBuffer,
    sun_ibo: GLBuffer,
    sun_index_count: i32,
    
    // Instanced rendering
    instance_buffer: InstanceBuffer,
    use_instancing: bool,
    
    // Skybox and clouds
    skybox: Skybox,
    clouds: CloudSystem,
    cloud_mesh_vbo: GLBuffer,
    cloud_mesh_ibo: GLBuffer,
    cloud_mesh_index_count: i32,
    
    // LOD system
    lod_manager: LODManager,
    
    // Worker pool for multi-threading
    worker_pool: WorkerPool,
    use_workers: bool,
    
    // Multiplayer networking
    network: NetworkManager,
    multiplayer_enabled: bool,
    
    // Player physics
    player_rigidbody: Rigidbody,
    player_height: f32,
    player_can_jump: bool,
    
    // Ground bounds (players fall off if outside)
    ground_size: f32,
    
    // Box mesh for pickable objects
    box_vbo: GLBuffer,
    box_ibo: GLBuffer,
    box_index_count: i32,
    
    // Shadow system
    shadow_system: ShadowSystem,
    shadows_enabled: bool,
    
    // Pointer lock state (only move camera when locked)
    pointer_locked: bool,
    
    // Track who last hit us (for kill credit)
    last_hit_by: Option<String>,
    
    // Particle settings
    pub spawn_rate: u32,
    pub particle_size: f32,
    pub particle_speed: f32,
    pub particle_lifetime: f32,
}

impl GameState {
    pub fn new() -> Result<Self, JsValue> {
        // Setup canvas
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let body = document.body().expect("no body");

        let canvas = document
            .create_element("canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        
        // Make canvas responsive
        let w = window.inner_width()?.as_f64().unwrap_or(800.0) as u32;
        let h = window.inner_height()?.as_f64().unwrap_or(600.0) as u32;
        canvas.set_width(w);
        canvas.set_height(h);
        canvas.style().set_property("display", "block")?;
        canvas.style().set_property("width", "100%")?;
        canvas.style().set_property("height", "100vh")?;
        
        body.append_child(&canvas)?;

        // Initialize WebGL context
        let gl_context = WebGLContext::new(canvas.clone())?;
        let gl = &gl_context.gl;

        // Create shaders (instanced version)
        let shader_program = ShaderProgram::new(gl, INSTANCED_VERTEX_SHADER, INSTANCED_FRAGMENT_SHADER)
            .map_err(|e| JsValue::from_str(&e))?;

        // Create particle mesh (simple cube)
        let particle_mesh = Mesh::cube(0.05);
        let vertex_data = vertex_data_interleaved(&particle_mesh.vertices);
        
        // Create vertex buffer
        let particle_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        particle_vbo.set_data(gl, &vertex_data, GL::STATIC_DRAW);

        // Create index buffer
        let particle_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        particle_ibo.set_data_u16(gl, &particle_mesh.indices, GL::STATIC_DRAW);
        let particle_index_count = particle_mesh.indices.len() as i32;

        // Create ground mesh as a THICK SLAB (cube with proper normals on all faces)
        let ground_mesh = Mesh::cube(1.0);  // Unit cube, will be scaled to ground size
        let ground_vertex_data = vertex_data_interleaved(&ground_mesh.vertices);
        let ground_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        ground_vbo.set_data(gl, &ground_vertex_data, GL::STATIC_DRAW);
        
        let ground_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        ground_ibo.set_data_u16(gl, &ground_mesh.indices, GL::STATIC_DRAW);
        let ground_index_count = ground_mesh.indices.len() as i32;
        
        // Ground collision plane
        let ground_plane = Plane::from_point_normal(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        
        // Ground size for collision detection (players fall off edges)
        // 50 units from center = 100x100 total area (small enough to walk off!)
        let ground_size = 100.0;  // Bigger platform (200x200 total)
        
        // Create player mesh (pill/capsule shape)
        let (player_vertices, player_indices) = PlayerMesh::capsule(0.4, 1.0, 16, 8);
        let player_vertex_data = vertex_data_interleaved(&player_vertices);
        let player_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        player_vbo.set_data(gl, &player_vertex_data, GL::STATIC_DRAW);
        
        let player_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        player_ibo.set_data_u16(gl, &player_indices, GL::STATIC_DRAW);
        let player_index_count = player_indices.len() as i32;
        
        // Create sun mesh (bright yellow sphere)
        let (sun_vertices, sun_indices) = SunMesh::sun(5.0, 16);
        let sun_vertex_data = vertex_data_interleaved(&sun_vertices);
        let sun_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        sun_vbo.set_data(gl, &sun_vertex_data, GL::STATIC_DRAW);
        
        let sun_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        sun_ibo.set_data_u16(gl, &sun_indices, GL::STATIC_DRAW);
        let sun_index_count = sun_indices.len() as i32;

        // Create skybox with SLOW day/night cycle
        let mut skybox = Skybox::new(gl)?;
        skybox.cycle_speed = 0.002;  // Very slow - ~8 minute full cycle
        
        // Create shadow system for realistic shadows
        let shadow_system = ShadowSystem::new(gl)?;
        web_sys::console::log_1(&"🌑 Shadow system initialized!".into());
        
        // Create clouds (20 voxel/Minecraft style clouds) - fallback for solo mode
        let clouds = CloudSystem::new(20, 100.0);
        
        // Create box mesh for pickable objects - COMPANION CUBE from GLTF!
        let box_mesh = crate::assets::companion_cube();  // Loaded from GLTF at compile time
        let box_vertex_data = vertex_data_interleaved(&box_mesh.vertices);
        let box_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        box_vbo.set_data(gl, &box_vertex_data, GL::STATIC_DRAW);
        
        let box_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        box_ibo.set_data_u16(gl, &box_mesh.indices, GL::STATIC_DRAW);
        let box_index_count = box_mesh.indices.len() as i32;
        
        // Create cloud mesh (CUBE for Minecraft/voxel look)
        let cloud_mesh = Mesh::cube(1.0);
        let cloud_vertex_data = vertex_data_interleaved(&cloud_mesh.vertices);
        let cloud_mesh_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        cloud_mesh_vbo.set_data(gl, &cloud_vertex_data, GL::STATIC_DRAW);
        
        let cloud_mesh_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        cloud_mesh_ibo.set_data_u16(gl, &cloud_mesh.indices, GL::STATIC_DRAW);
        let cloud_mesh_index_count = cloud_mesh.indices.len() as i32;

        // Create instance buffer (1 million particles capacity)
        let instance_buffer = InstanceBuffer::new(gl, 1_000_000)?;

        // Setup vertex attributes
        shader_program.use_program(gl);
        
        particle_vbo.bind(gl);
        
        let stride = 12 * 4; // 12 floats * 4 bytes
        
        // Per-vertex attributes (not instanced)
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
        
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
        
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
        
        gl.enable_vertex_attrib_array(3);
        gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
        
        // Setup per-instance attributes
        instance_buffer.setup_attributes(gl);

        // Setup camera
        let aspect = gl_context.get_aspect_ratio();
        // Start camera at TOP of pill (player_height above ground)
        // player_height is 5.0, so camera at y = 5.0
        let camera = Camera::new(
            Vec3::new(0.0, 5.0, 10.0),  // Camera at top of pill
            Vec3::new(0.0, 5.0, 0.0),   // Look at center
            deg_to_rad(45.0),
            aspect,
        );

        // Create scene with 1 million particle capacity
        let scene = Scene::new(1_000_000);

        // Setup mouse events
        setup_mouse_events(&canvas)?;

        // Initialize LOD system
        let lod_manager = LODManager::new();
        
        // Initialize worker pool (detect CPU cores)
        let worker_count = WorkerPool::optimal_worker_count();
        let worker_pool = WorkerPool::new(worker_count);
        
        web_sys::console::log_1(&format!("💻 Detected {} CPU cores for workers", worker_count).into());
        
        // Initialize player physics
        let player_rigidbody = Rigidbody::new().with_drag(0.9);
        let player_height = 5.0;  // Player is 5 units tall - camera at TOP of pill

        Ok(Self {
            engine: Engine::new(),
            gl_context,
            camera,
            scene,
            input: Input::new(),
            shader_program,
            particle_vbo,
            particle_ibo,
            particle_index_count,
            ground_vbo,
            ground_ibo,
            ground_index_count,
            ground_plane,
            player_vbo,
            player_ibo,
            player_index_count,
            sun_vbo,
            sun_ibo,
            sun_index_count,
            instance_buffer,
            use_instancing: true,
            skybox,
            clouds,
            cloud_mesh_vbo,
            cloud_mesh_ibo,
            cloud_mesh_index_count,
            lod_manager,
            worker_pool,
            use_workers: false,  // Can enable later with actual Web Workers
            network: NetworkManager::new(),
            multiplayer_enabled: false,
            player_rigidbody,
            player_height,
            player_can_jump: true,
            ground_size,
            box_vbo,
            box_ibo,
            box_index_count,
            shadow_system,
            shadows_enabled: true,  // Re-enabled with fixes
            pointer_locked: false,  // Start unlocked until player clicks PLAY
            last_hit_by: None,  // Track who hit us for kill credit
            spawn_rate: 50,
            particle_size: 0.14,
            particle_speed: 3.2,
            particle_lifetime: 2.5,
        })
    }

    pub fn update(&mut self, input: &Input) {
        let dt = self.engine.delta_time;
        
        // Update LOD system
        self.lod_manager.update(&self.camera);
        
        // Update skybox (day/night cycle)
        self.skybox.update(dt);
        
        // Update clouds
        self.clouds.update(dt);
        
        // Update scene
        self.scene.update(&self.engine);

        // Player movement (WASD controls with physics)
        let move_force = 60.0;  // Good movement speed
        let jump_force = 28.0;  // High jump - can easily jump on boxes!
        
        // Sprint with SHIFT - 2x speed!
        let is_sprinting = input.keyboard.is_pressed(input::KEY_SHIFT);
        let actual_force = if is_sprinting { move_force * 2.0 } else { move_force };
        
        if input.keyboard.is_pressed(input::KEY_W) {
            let force = self.camera.forward * actual_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_S) {
            let force = self.camera.forward * -actual_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_A) {
            let force = self.camera.right * -actual_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_D) {
            let force = self.camera.right * actual_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        
        // Jump (only when grounded)
        if input.keyboard.is_just_pressed(input::KEY_SPACE) && self.player_can_jump {
            self.player_rigidbody.apply_impulse(Vec3::new(0.0, jump_force, 0.0));
            self.player_can_jump = false;
        }
        
        // E key for pickup/drop (multiplayer)
        let is_holding = self.network.is_holding_object();
        // Use a position near player's "hands" (lower than camera) for pickup detection
        let pickup_check_pos = Vec3::new(
            self.camera.position.x,
            self.camera.position.y - 3.0,  // Hands are lower than camera (eyes)
            self.camera.position.z,
        );
        let near_pickable = self.network.get_nearest_pickable(pickup_check_pos, 5.0);
        
        // Update UI prompts
        if self.multiplayer_enabled {
            showPickupPrompt(near_pickable.is_some() && !is_holding);
            showHeldItem(is_holding);
        }
        
        if input.keyboard.is_just_pressed(input::KEY_E) && self.multiplayer_enabled {
            if is_holding {
                // Drop the object gently in front of player (at camera height - 2)
                let drop_pos = self.camera.position + self.camera.forward * 3.0;
                let drop_velocity = Vec3::ZERO; // Gentle drop
                let _ = self.network.send_drop_object_with_velocity(
                    Vec3::new(drop_pos.x, self.camera.position.y - 2.0, drop_pos.z),
                    drop_velocity,
                );
            } else {
                // Try to pick up nearest object within 4 units
                if let Some(obj_id) = near_pickable {
                    let _ = self.network.send_pickup_object(obj_id);
                }
            }
        }
        
        // LEFT MOUSE CLICK to THROW object
        if input.mouse.is_just_pressed(0) && is_holding && self.multiplayer_enabled {
            // Throw the object with velocity in the direction we're looking - POWERFUL THROW (x4)
            let throw_speed = 80.0;  // x4 throw distance
            let throw_velocity = self.camera.forward * throw_speed + Vec3::new(0.0, 15.0, 0.0); // Higher arc
            let throw_pos = self.camera.position + self.camera.forward * 3.0;
            let _ = self.network.send_drop_object_with_velocity(
                Vec3::new(throw_pos.x, self.camera.position.y, throw_pos.z),
                throw_velocity,
            );
        }

        // Update player physics
        self.player_rigidbody.update(dt, GRAVITY);
        
        // Apply physics to camera position
        let displacement = self.player_rigidbody.get_displacement(dt);
        self.camera.position += displacement;
        self.camera.target += displacement;

        // Ground surface is at y=0, ground extends from -ground_size to +ground_size on X and Z
        // Camera is at TOP of pill, so feet are at camera.y - player_height
        let feet_y = self.camera.position.y - self.player_height;
        
        // Check if player is OVER the green floor (within bounds on X and Z)
        let over_green_floor = self.camera.position.x.abs() <= self.ground_size 
                            && self.camera.position.z.abs() <= self.ground_size;
        
        // Check if player is standing on a BOX
        // Boxes are 2x2x2, so top surface is at box.position.y + 1.0
        let box_size = 2.0;
        let box_half = box_size / 2.0;
        let mut standing_on_box = false;
        let mut box_top_y = 0.0f32;
        
        // Get box positions
        let box_positions: Vec<Vec3> = if self.multiplayer_enabled {
            self.network.pickables.iter()
                .filter(|p| p.held_by.is_none())  // Only check boxes NOT being held
                .map(|p| p.position)
                .collect()
        } else {
            vec![Vec3::new(5.0, 1.0, 5.0)]
        };
        
        for box_pos in &box_positions {
            // Check if player is within X/Z bounds of box
            let in_box_xz = (self.camera.position.x - box_pos.x).abs() < box_half + 0.5
                         && (self.camera.position.z - box_pos.z).abs() < box_half + 0.5;
            
            if in_box_xz {
                let this_box_top = box_pos.y + box_half;
                // Only land on box if:
                // 1. Falling down (velocity < 0)
                // 2. Feet are near or below box top
                // 3. Feet are above box bottom (not under the box)
                let is_falling = self.player_rigidbody.velocity.y <= 0.0;
                let feet_near_top = feet_y <= this_box_top + 0.5 && feet_y >= this_box_top - 0.5;
                let feet_above_box = feet_y > box_pos.y;  // Above box center = on top, not under
                
                if is_falling && feet_near_top && feet_above_box {
                    if this_box_top > box_top_y {
                        box_top_y = this_box_top;
                        standing_on_box = true;
                    }
                }
            }
        }
        
        if standing_on_box {
            // Land on the box!
            self.camera.position.y = box_top_y + self.player_height;
            self.camera.target.y = self.camera.position.y + self.camera.forward.y;
            self.player_rigidbody.velocity.y = 0.0;
            self.player_rigidbody.is_grounded = true;
            self.player_can_jump = true;
        } else if over_green_floor {
            // Player is above green floor area
            if feet_y <= 0.0 {
                // Feet are at or below ground level - land on ground!
                self.camera.position.y = self.player_height;  // Camera at top, feet at y=0
                self.camera.target.y = self.camera.position.y + self.camera.forward.y;
                self.player_rigidbody.velocity.y = 0.0;
                self.player_rigidbody.is_grounded = true;
                self.player_can_jump = true;
            } else {
                // In the air above ground (jumping/falling)
                self.player_rigidbody.is_grounded = false;
            }
        } else {
            // Player walked off the green floor - they're falling into the void!
            self.player_rigidbody.is_grounded = false;
            self.player_can_jump = false;
            
            // Respawn if fallen too far
            if self.camera.position.y < -30.0 {
                // Report death to server (resets score)
                if self.multiplayer_enabled {
                    let _ = self.network.send_report_death();
                    
                    // Check if we were recently hit by someone - give them a point!
                    if let Some(ref killer_id) = self.last_hit_by {
                        let _ = self.network.send_report_kill(killer_id.clone());
                        web_sys::console::log_1(&format!("💀 Killed by player {}!", killer_id).into());
                    }
                    self.last_hit_by = None;  // Reset
                }
                
                // Respawn at center of green floor, standing on top
                self.camera.position = Vec3::new(0.0, self.player_height, 0.0);
                self.camera.target = Vec3::new(0.0, self.player_height, -1.0);
                self.player_rigidbody.velocity = Vec3::ZERO;
                self.player_rigidbody.is_grounded = true;
                self.player_can_jump = true;
                web_sys::console::log_1(&"🔄 Respawned on top of the green floor!".into());
            }
        }

        // HORIZONTAL box collision - can't walk through boxes!
        // If box is moving fast (thrown), it pushes player HARD
        let player_radius = 0.5;  // Player collision radius
        for pickable in &self.network.pickables {
            let box_pos = pickable.position;
            let box_vel = pickable.velocity;
            
            // Skip if held by someone
            if pickable.held_by.is_some() {
                continue;
            }
            
            // Check vertical overlap (player body overlaps with box height)
            let player_bottom = self.camera.position.y - self.player_height;
            let player_top = self.camera.position.y;
            let box_bottom = box_pos.y - box_half;
            let box_top = box_pos.y + box_half;
            
            let vertical_overlap = player_bottom < box_top && player_top > box_bottom;
            
            if vertical_overlap {
                // Check horizontal distance
                let dx = self.camera.position.x - box_pos.x;
                let dz = self.camera.position.z - box_pos.z;
                
                // Push player out if inside box bounds
                let push_dist = box_half + player_radius;
                
                if dx.abs() < push_dist && dz.abs() < push_dist {
                    // Calculate box speed for impact force - MEGA HIT!
                    let box_speed = (box_vel.x * box_vel.x + box_vel.z * box_vel.z).sqrt();
                    let impact_multiplier = if box_speed > 3.0 { 
                        20.0 + box_speed * 3.0  // MASSIVE knockback from thrown boxes!
                    } else { 
                        1.0  // Stationary box just blocks
                    };
                    
                    // Inside box - push out along shortest axis
                    let overlap_x = push_dist - dx.abs();
                    let overlap_z = push_dist - dz.abs();
                    
                    // Track who threw this box at us (for kill credit)
                    if box_speed > 3.0 {
                        // Find who was holding this box recently - check all players
                        for player in &self.network.remote_players {
                            // If a player is close to where the box came from, credit them
                            let player_to_box = ((player.position.x - box_pos.x).powi(2) + 
                                                 (player.position.z - box_pos.z).powi(2)).sqrt();
                            if player_to_box < 15.0 {
                                self.last_hit_by = Some(player.id.clone());
                                break;
                            }
                        }
                    }
                    
                    if overlap_x < overlap_z {
                        // Push along X
                        let push = if dx > 0.0 { overlap_x * impact_multiplier } else { -overlap_x * impact_multiplier };
                        self.camera.position.x += push;
                        self.camera.target.x += push;
                        // Add MASSIVE velocity push from thrown box - sends them FLYING!
                        if box_speed > 3.0 {
                            self.player_rigidbody.apply_impulse(Vec3::new(box_vel.x * 4.0, 15.0, box_vel.z * 4.0));
                        }
                    } else {
                        // Push along Z
                        let push = if dz > 0.0 { overlap_z * impact_multiplier } else { -overlap_z * impact_multiplier };
                        self.camera.position.z += push;
                        self.camera.target.z += push;
                        // Add MASSIVE velocity push from thrown box - sends them FLYING!
                        if box_speed > 3.0 {
                            self.player_rigidbody.apply_impulse(Vec3::new(box_vel.x * 4.0, 15.0, box_vel.z * 4.0));
                        }
                    }
                }
            }
        }

        // If holding a box, check collision with other players - box stops at collision!
        if self.network.is_holding_object() && self.multiplayer_enabled {
            let held_box_pos = self.camera.position + self.camera.forward * 2.5;
            let box_half = 1.0;
            let player_radius = 1.5;  // Other player collision radius
            
            for remote_player in &self.network.remote_players {
                // Skip self
                if let Some(ref my_id) = self.network.player_id {
                    if &remote_player.id == my_id {
                        continue;
                    }
                }
                
                // Check collision between held box and remote player
                let dx = held_box_pos.x - remote_player.position.x;
                let dz = held_box_pos.z - remote_player.position.z;
                let dist = (dx * dx + dz * dz).sqrt();
                
                let collision_dist = box_half + player_radius;
                if dist < collision_dist && dist > 0.01 {
                    // Push ourselves back (can't push through player with box)
                    let push_amount = collision_dist - dist;
                    let push_dir_x = dx / dist;
                    let push_dir_z = dz / dist;
                    
                    self.camera.position.x -= push_dir_x * push_amount;
                    self.camera.position.z -= push_dir_z * push_amount;
                    self.camera.target.x -= push_dir_x * push_amount;
                    self.camera.target.z -= push_dir_z * push_amount;
                }
            }
        }

        // Mouse look - only when pointer is locked (cursor hidden)
        if self.pointer_locked {
            let sensitivity = 0.002;
            self.camera.rotate(
                input.mouse.delta.x * sensitivity,
                -input.mouse.delta.y * sensitivity,
            );
        }
        
        // Apply gravity and GROUND BOUNCE to particles
        // This could be parallelized with Web Workers for even better performance!
        let _entity_count = self.scene.entities.entities.len();
        
        // Get ALL box positions for particle collision (including held boxes for visual effect)
        let particle_box_positions: Vec<Vec3> = if self.multiplayer_enabled {
            self.network.pickables.iter().map(|p| p.position).collect()
        } else {
            vec![Vec3::new(5.0, 1.0, 5.0)]  // Solo mode box
        };
        
        // Get ALL player positions for collision (including LOCAL player!)
        let mut player_positions: Vec<Vec3> = self.network.remote_players.iter()
            .filter(|p| {
                // Don't collide with self in the remote list
                if let Some(ref my_id) = self.network.player_id {
                    &p.id != my_id
                } else {
                    true
                }
            })
            .map(|p| Vec3::new(p.position.x, 1.8, p.position.z))  // Pill center y=1.8
            .collect();
        
        // Add LOCAL player position for particle collision!
        // Camera is at TOP of pill, pill center is at camera.y - player_height/2
        let local_pill_center_y = self.camera.position.y - self.player_height / 2.0;
        player_positions.push(Vec3::new(
            self.camera.position.x,
            local_pill_center_y,
            self.camera.position.z,
        ));
        
        let ground_bounds = self.ground_size;
        
        // Particles to remove (fell too far)
        let mut entities_to_remove = Vec::new();
        
        // Single-threaded physics for particles
        for &entity in &self.scene.entities.entities.clone() {
            if let (Some(transform), Some(velocity)) = (
                self.scene.entities.transforms.get_mut(entity),
                self.scene.entities.velocities.get_mut(entity),
            ) {
                // Apply gravity
                velocity.linear += GRAVITY * dt;
                
                let pos = &mut transform.position;
                let vel = &mut velocity.linear;
                
                // Update position based on velocity
                *pos += *vel * dt;
                
                // Check if particle is OVER the green floor (within bounds)
                let over_green_floor = pos.x.abs() <= ground_bounds && pos.z.abs() <= ground_bounds;
                
                // FIRST: Check collision with boxes (they sit on the ground)
                let mut hit_box = false;
                for box_pos in &particle_box_positions {
                    let box_half = 1.0; // Box is 2x2x2, half size is 1
                    
                    // Check if inside box bounds
                    if pos.x >= box_pos.x - box_half && pos.x <= box_pos.x + box_half &&
                       pos.y >= box_pos.y - box_half && pos.y <= box_pos.y + box_half &&
                       pos.z >= box_pos.z - box_half && pos.z <= box_pos.z + box_half {
                        
                        hit_box = true;
                        
                        // Find which face we're closest to and bounce off it
                        let dx = pos.x - box_pos.x;
                        let dy = pos.y - box_pos.y;
                        let dz = pos.z - box_pos.z;
                        
                        // Normalize to find penetration on each axis
                        let px = box_half - dx.abs();
                        let py = box_half - dy.abs();
                        let pz = box_half - dz.abs();
                        
                        // Push out along axis with least penetration
                        if px < py && px < pz {
                            pos.x = box_pos.x + box_half * dx.signum() * 1.01;
                            vel.x = -vel.x * 0.6;
                        } else if py < pz {
                            pos.y = box_pos.y + box_half * dy.signum() * 1.01;
                            vel.y = -vel.y * 0.6;
                        } else {
                            pos.z = box_pos.z + box_half * dz.signum() * 1.01;
                            vel.z = -vel.z * 0.6;
                        }
                    }
                }
                
                // SECOND: Check collision with remote players (cylinders)
                for player_pos in &player_positions {
                    let player_radius = 1.2;
                    let player_bottom = 0.0;  // Feet at ground
                    let player_top = 3.6;     // Head height
                    
                    // Check if within cylinder height
                    if pos.y >= player_bottom && pos.y <= player_top {
                        let dx = pos.x - player_pos.x;
                        let dz = pos.z - player_pos.z;
                        let horizontal_dist = (dx * dx + dz * dz).sqrt();
                        
                        if horizontal_dist < player_radius {
                            // Inside player cylinder - push out and bounce
                            let push_dir = if horizontal_dist > 0.01 {
                                (dx / horizontal_dist, dz / horizontal_dist)
                            } else {
                                (1.0, 0.0)
                            };
                            
                            pos.x = player_pos.x + push_dir.0 * player_radius * 1.01;
                            pos.z = player_pos.z + push_dir.1 * player_radius * 1.01;
                            vel.x = push_dir.0 * vel.x.abs() * 0.8 + push_dir.0 * 2.0;
                            vel.z = push_dir.1 * vel.z.abs() * 0.8 + push_dir.1 * 2.0;
                        }
                    }
                }
                
                // THIRD: Ground collision - only if over the green floor AND not already hit something
                if !hit_box && over_green_floor && pos.y <= 0.0 {
                    pos.y = 0.01; // Slightly above ground
                    vel.y = -vel.y * 0.6; // Bounce
                    vel.x *= 0.92; // Friction
                    vel.z *= 0.92;
                }
                
                // Remove particles that fell into the void
                if pos.y < -50.0 {
                    entities_to_remove.push(entity);
                }
            }
        }
        
        // Remove fallen particles
        for entity in entities_to_remove {
            self.scene.entities.destroy_entity(entity);
        }

        // Spawn particles continuously based on spawn rate
        if self.spawn_rate > 0 && self.engine.frame_count % 1 == 0 {
            self.spawn_particle_burst(self.spawn_rate);
        }
        
        // Multiplayer: sync state and send updates
        if self.multiplayer_enabled {
            // Sync state from WebSocket callbacks
            self.network.sync_state();
            
            // Smooth interpolation for all server-synced data (0.15 = smooth but responsive)
            self.network.interpolate(0.15);
            
            // Send player position to server (every 3 frames = 20 Hz)
            if self.engine.frame_count % 3 == 0 {
                let _ = self.network.send_player_update(
                    self.camera.position,
                    Vec3::new(self.camera.yaw, self.camera.pitch, 0.0),
                    Vec3::ZERO,
                );
            }
            
            // Sync time of day with server
            self.skybox.time_of_day = self.network.server_time;
            
            // Rain is now spawned SERVER-SIDE and synced via server particles
            // All players see the same rain drops falling from the same clouds!
        }

        // Log FPS and LOD stats
        if self.engine.frame_count % 60 == 0 {
            let stats = &self.lod_manager.stats;
            web_sys::console::log_1(&format!(
                "FPS: {:.1} | Total: {} | Visible: {} | Culled: {} (Frustum: {}, Distance: {}) | LOD: H:{} M:{} L:{} VL:{}",
                self.engine.fps,
                stats.total_objects,
                stats.visible_objects,
                stats.culled_by_frustum + stats.culled_by_distance,
                stats.culled_by_frustum,
                stats.culled_by_distance,
                stats.high_lod,
                stats.medium_lod,
                stats.low_lod,
                stats.very_low_lod,
            ).into());
        }
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let gl = &self.gl_context.gl;
        
        // Get viewport size
        let (width, height) = self.gl_context.get_viewport_size();
        
        // Get sun direction for lighting and shadows
        let sun_dir = self.skybox.get_sun_direction();
        let _is_night = self.skybox.is_night();  // Kept for future use
        
        // ============================================
        // SHADOW PASS - Render scene from sun's view
        // ============================================
        // Only render shadows when sun is above horizon (y > 0.1)
        if self.shadows_enabled && sun_dir.y > 0.1 {
            // Update light space matrix based on sun position
            let scene_center = Vec3::ZERO;
            let scene_radius = self.ground_size * 1.5;
            self.shadow_system.update_light_matrix(sun_dir, scene_center, scene_radius);
            
            // Begin shadow pass
            self.shadow_system.begin_shadow_pass(gl);
            
            // Set light space matrix uniform for depth shader
            if let Some(loc) = self.shadow_system.get_depth_uniform(gl, "u_lightSpaceMatrix") {
                gl.uniform_matrix4fv_with_f32_array(
                    Some(&loc), 
                    false, 
                    &self.shadow_system.light_space_matrix.as_array()
                );
            }
            
            // Setup instance buffer attributes for shadow pass
            self.instance_buffer.setup_attributes(gl);
            
            // Render shadow casters (boxes, players - NOT ground, it receives shadows!)
            // Ground is NOT rendered to shadow map - it should RECEIVE shadows, not cast them
            let stride = 12 * 4;
            
            // Render boxes to shadow map
            self.box_vbo.bind(gl);
            gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
            self.box_ibo.bind(gl);
            
            let box_instances: Vec<InstanceData> = if self.multiplayer_enabled {
                self.network.pickables.iter()
                    .map(|obj| InstanceData::new(obj.position, Vec3::new(2.0, 2.0, 2.0), Color::rgb(1.0, 1.0, 1.0)))
                    .collect()
            } else {
                vec![InstanceData::new(Vec3::new(5.0, 1.0, 5.0), Vec3::new(2.0, 2.0, 2.0), Color::rgb(1.0, 1.0, 1.0))]
            };
            
            if !box_instances.is_empty() {
                self.instance_buffer.update(gl, &box_instances);
                gl.draw_elements_instanced_with_i32(GL::TRIANGLES, self.box_index_count, GL::UNSIGNED_SHORT, 0, box_instances.len() as i32);
            }
            
            // Render players to shadow map (including LOCAL player for shadow!)
            self.player_vbo.bind(gl);
            gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
            self.player_ibo.bind(gl);
            
            let mut player_instances: Vec<InstanceData> = self.network.remote_players.iter()
                .filter(|p| {
                    if let Some(ref my_id) = self.network.player_id {
                        &p.id != my_id
                    } else { true }
                })
                .map(|p| {
                    let player_height = 5.0;  // Camera at top of pill
                    let base_scale = Vec3::new(2.0, 2.5, 2.0);
                    let base_height = 1.8 * base_scale.y;  // Visual pill height
                    let half_height = base_height / 2.0;
                    // Camera is at TOP, so feet = camera.y - player_height
                    let feet_y = p.position.y - player_height;
                    let pill_center_y = feet_y + half_height;
                    InstanceData::new(Vec3::new(p.position.x, pill_center_y, p.position.z), base_scale, Color::rgb(1.0, 1.0, 1.0))
                })
                .collect();
            
            // Add LOCAL player to shadow casters (you can see your own shadow!)
            {
                let local_player_height = self.player_height;
                let base_scale = Vec3::new(2.0, 2.5, 2.0);
                let base_height = 1.8 * base_scale.y;
                let half_height = base_height / 2.0;
                let feet_y = self.camera.position.y - local_player_height;
                let pill_center_y = feet_y + half_height;
                player_instances.push(InstanceData::new(
                    Vec3::new(self.camera.position.x, pill_center_y, self.camera.position.z),
                    base_scale,
                    Color::rgb(1.0, 1.0, 1.0),
                ));
            }
            
            if !player_instances.is_empty() {
                self.instance_buffer.update(gl, &player_instances);
                gl.draw_elements_instanced_with_i32(GL::TRIANGLES, self.player_index_count, GL::UNSIGNED_SHORT, 0, player_instances.len() as i32);
            }
            
            // End shadow pass
            self.shadow_system.end_shadow_pass(gl, width as i32, height as i32);
        }
        
        // ============================================
        // MAIN PASS - Render scene with shadows
        // ============================================
        
        // Update viewport for main pass
        gl.viewport(0, 0, width as i32, height as i32);
        
        // Clear screen with skybox color
        let sky_color = self.skybox.get_sky_color();
        self.gl_context.clear(sky_color.x, sky_color.y, sky_color.z, 1.0);

        // Use main shader
        self.shader_program.use_program(gl);

        // Update camera aspect ratio
        self.camera.aspect = self.gl_context.get_aspect_ratio();
        let view_proj = self.camera.view_projection_matrix();

        // Get uniform locations
        let u_view_proj = self.shader_program.get_uniform_location(gl, "u_viewProj")
            .ok_or("Failed to get u_viewProj location")?;
        let u_sun_direction = self.shader_program.get_uniform_location(gl, "u_sunDirection");
        let u_sun_color = self.shader_program.get_uniform_location(gl, "u_sunColor");
        let u_ambient_strength = self.shader_program.get_uniform_location(gl, "u_ambientStrength");
        let u_light_space_matrix = self.shader_program.get_uniform_location(gl, "u_lightSpaceMatrix");
        let u_shadow_map = self.shader_program.get_uniform_location(gl, "u_shadowMap");
        let u_shadows_enabled = self.shader_program.get_uniform_location(gl, "u_shadowsEnabled");

        // Set view-projection matrix
        gl.uniform_matrix4fv_with_f32_array(Some(&u_view_proj), false, &view_proj.as_array());
        
        // Set light space matrix for shadow lookup
        if let Some(loc) = u_light_space_matrix {
            gl.uniform_matrix4fv_with_f32_array(Some(&loc), false, &self.shadow_system.light_space_matrix.as_array());
        }
        
        // Bind shadow map texture only when shadows are enabled AND sun is above horizon
        // Use sun_dir.y > 0.1 to check if sun is meaningfully above horizon
        let shadows_active = self.shadows_enabled && sun_dir.y > 0.1;
        if let Some(loc) = u_shadow_map {
            if shadows_active {
                self.shadow_system.bind_shadow_map(gl, 0);
            } else {
                // Bind null texture when shadows disabled
                gl.active_texture(GL::TEXTURE0);
                gl.bind_texture(GL::TEXTURE_2D, None);
            }
            gl.uniform1i(Some(&loc), 0);
        }
        
        // Enable/disable shadows
        if let Some(ref loc) = u_shadows_enabled {
            gl.uniform1i(Some(loc), if shadows_active { 1 } else { 0 });
        }
        
        // Set sun lighting uniforms
        if let Some(ref loc) = u_sun_direction {
            gl.uniform3f(Some(loc), sun_dir.x, sun_dir.y, sun_dir.z);
        }
        
        // Calculate smooth day/night factor (0 = night, 1 = day)
        // Smooth transition instead of instant flip
        let t = self.skybox.time_of_day;
        let day_factor = if t < 0.2 {
            // Early night -> dawn transition
            let dawn_t = (t - 0.15).max(0.0) / 0.05;
            dawn_t.clamp(0.0, 1.0)
        } else if t < 0.25 {
            // Dawn -> day
            let morning_t = (t - 0.2) / 0.05;
            morning_t.clamp(0.0, 1.0)
        } else if t < 0.75 {
            // Full day
            1.0
        } else if t < 0.8 {
            // Dusk -> night
            let dusk_t = 1.0 - (t - 0.75) / 0.05;
            dusk_t.clamp(0.0, 1.0)
        } else {
            // Night
            0.0
        };
        
        // Smooth interpolation using smoothstep
        let smooth_factor = day_factor * day_factor * (3.0 - 2.0 * day_factor);
        
        // Interpolate sun color between moonlight and sunlight
        let night_color = Vec3::new(0.3, 0.3, 0.5);  // Moonlight (bluish)
        let day_color = Vec3::new(1.0, 0.95, 0.8);    // Sunlight (warm yellow)
        let sun_color = night_color.lerp(&day_color, smooth_factor);
        
        if let Some(ref loc) = u_sun_color {
            gl.uniform3f(Some(loc), sun_color.x, sun_color.y, sun_color.z);
        }
        
        // Smooth ambient transition too
        let ambient = 0.15 + 0.15 * smooth_factor;  // 0.15 at night, 0.3 at day
        if let Some(ref loc) = u_ambient_strength {
            gl.uniform1f(Some(loc), ambient);
        }

        // Render clouds first (in background, with transparency)
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        
        let is_night = self.skybox.is_night();
        
        // Use server-synced clouds if multiplayer, otherwise local clouds
        let cloud_instances: Vec<InstanceData> = if self.multiplayer_enabled && !self.network.server_clouds.is_empty() {
            // Server-synced clouds (same positions for all players!)
            self.network.server_clouds.iter()
                .map(|cloud| {
                    let color = if is_night {
                        Color::new(0.3, 0.3, 0.4, 0.9)
                    } else {
                        Color::new(0.95, 0.95, 0.95, 0.9)
                    };
                    InstanceData::new(
                        cloud.position,
                        Vec3::splat(cloud.scale),
                        color,
                    )
                })
                .collect()
        } else {
            // Fallback to local clouds (solo mode)
            self.clouds.get_instances(is_night)
                .iter()
                .map(|(pos, scale, color)| InstanceData::new(*pos, *scale, *color))
                .collect()
        };
        
        // Vertex stride for all meshes (12 floats * 4 bytes = 48)
        let stride = 12 * 4;
        
        if !cloud_instances.is_empty() {
            // Setup cloud mesh vertex attributes
            self.cloud_mesh_vbo.bind(gl);
            gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
            gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
            gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
            gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
            
            self.cloud_mesh_ibo.bind(gl);
            self.instance_buffer.update(gl, &cloud_instances);
            
            gl.draw_elements_instanced_with_i32(
                GL::TRIANGLES,
                self.cloud_mesh_index_count,
                GL::UNSIGNED_SHORT,
                0,
                cloud_instances.len() as i32,
            );
        }
        
        gl.disable(GL::BLEND);
        
        // Draw sun - only when above the horizon!
        let sun_dir = self.skybox.get_sun_direction();
        
        // Only render sun when it's above the horizon (sun_dir.y > 0)
        if sun_dir.y > 0.0 {
            // Keep depth test ON so clouds can occlude the sun
            // Temporarily disable shadows and set full brightness for sun
            if let Some(ref loc) = u_shadows_enabled {
                gl.uniform1i(Some(loc), 0);  // No shadows on sun
            }
            if let Some(ref loc) = u_ambient_strength {
                gl.uniform1f(Some(loc), 1.0);  // Full brightness - no lighting calculations
            }
            
            let sun_distance = 300.0;  // Far away
            let sun_position = self.camera.position + sun_dir * sun_distance;
            
            let sun_instance = vec![InstanceData::new(
                sun_position,
                Vec3::splat(5.0),  // Smaller sun
                Color::rgb(1.0, 1.0, 0.0),  // PURE YELLOW
            )];
            
            // Setup sun mesh vertex attributes
            self.sun_vbo.bind(gl);
            gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
            gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
            gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
            gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
            
            self.sun_ibo.bind(gl);
            self.instance_buffer.update(gl, &sun_instance);
            
            gl.draw_elements_instanced_with_i32(
                GL::TRIANGLES,
                self.sun_index_count,
                GL::UNSIGNED_SHORT,
                0,
                1,
            );
            
            // Restore original lighting settings
            if let Some(ref loc) = u_shadows_enabled {
                gl.uniform1i(Some(loc), if shadows_active { 1 } else { 0 });
            }
            if let Some(ref loc) = u_ambient_strength {
                gl.uniform1f(Some(loc), ambient);
            }
        }
        
        // Draw local player (pill shape at camera position) - only if looking down/third person
        // For first person, we skip drawing our own pill
        // let player_instance = vec![InstanceData::new(
        //     self.camera.position,
        //     Vec3::ONE,
        //     Color::rgb(0.2, 0.5, 0.8),  // Blue player
        // )];
        // self.instance_buffer.update(gl, &player_instance);
        // self.player_vbo.bind(gl);
        // self.player_ibo.bind(gl);
        // gl.draw_elements_instanced_with_i32(
        //     GL::TRIANGLES,
        //     self.player_index_count,
        //     GL::UNSIGNED_SHORT,
        //     0,
        //     1,
        // );
        
        // Draw REMOTE players (other players from multiplayer) - BIG COLORFUL PILLS!
        let remote_players = &self.network.remote_players;
        if !remote_players.is_empty() {
            let mut remote_instances: Vec<InstanceData> = Vec::with_capacity(remote_players.len());
            
            for (i, player) in remote_players.iter().enumerate() {
                // Skip self (compare with our player ID)
                if let Some(ref my_id) = self.network.player_id {
                    if &player.id == my_id {
                        continue;
                    }
                }
                
                // Different BRIGHT color for each remote player
                let color = match i % 6 {
                    0 => Color::rgb(1.0, 0.2, 0.2),  // Bright Red
                    1 => Color::rgb(0.2, 1.0, 0.2),  // Bright Green
                    2 => Color::rgb(1.0, 1.0, 0.2),  // Bright Yellow
                    3 => Color::rgb(1.0, 0.2, 1.0),  // Bright Magenta
                    4 => Color::rgb(0.2, 1.0, 1.0),  // Bright Cyan
                    _ => Color::rgb(1.0, 0.6, 0.2),  // Bright Orange
                };
                
                // Player pill dimensions:
                // Base capsule: radius 0.4, height 1.0, total height = 1.8
                // We scale it to be BIGGER and more visible
                let base_scale = Vec3::new(2.0, 2.5, 2.0);  // BIG pill shape
                
                // Calculate pill height
                // Capsule total height after scale = 1.8 * 2.5 = 4.5
                // Half height = 2.25 (distance from center to feet)
                let base_height = 1.8 * base_scale.y;
                let half_height = base_height / 2.0;
                
                // Player position.y is their CAMERA position (at TOP of pill)
                // So feet_y = position.y - player_height
                // Pill center should be at feet_y + half_height
                let player_height = 5.0;  // Match our player height
                let feet_y = player.position.y - player_height;
                let pill_center_y = feet_y + half_height;
                
                // Smooth animation: Check if jumping/falling (feet above ground)
                let mut final_scale = base_scale;
                let is_in_air = feet_y > 0.1;
                
                if is_in_air {
                    // Stretch vertically when in air
                    final_scale.y *= 1.1;
                    final_scale.x *= 0.9;
                    final_scale.z *= 0.9;
                }
                
                // Smooth bobbing animation based on velocity (if moving on ground)
                let speed = (player.velocity.x * player.velocity.x + player.velocity.z * player.velocity.z).sqrt();
                let bob_amount = if speed > 0.1 && !is_in_air {
                    (self.engine.total_time * 10.0).sin() * 0.1
                } else {
                    0.0
                };
                
                remote_instances.push(InstanceData::new(
                    Vec3::new(player.position.x, pill_center_y + bob_amount, player.position.z),
                    final_scale,
                    color,
                ));
            }
            
            if !remote_instances.is_empty() {
                // Bind player mesh and setup vertex attributes
                self.player_vbo.bind(gl);
                gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
                gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
                gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
                gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
                
                self.player_ibo.bind(gl);
                self.instance_buffer.update(gl, &remote_instances);
                
                gl.draw_elements_instanced_with_i32(
                    GL::TRIANGLES,
                    self.player_index_count,
                    GL::UNSIGNED_SHORT,
                    0,
                    remote_instances.len() as i32,
                );
            }
        }
        
        // Draw ground (as thick slab with visible top and sides)
        let ground_color = if is_night {
            Color::rgb(0.05, 0.4, 0.05)   // Dark green at night
        } else {
            Color::rgb(0.2, 0.8, 0.2)   // BRIGHT GREEN grass during day
        };
        
        // Ground is a cube at y=-1 (so top surface is at y=0)
        // Scale: width/depth = ground_size*2, height = 2 (thick slab)
        let ground_instances = vec![InstanceData::new(
            Vec3::new(0.0, -1.0, 0.0),  // Center at y=-1 so top is at y=0
            Vec3::new(self.ground_size * 2.0, 2.0, self.ground_size * 2.0), // Big slab
            ground_color
        )];
        
        // Bind ground mesh and setup vertex attributes
        self.ground_vbo.bind(gl);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
        gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
        gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
        gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
        
        self.ground_ibo.bind(gl);
        self.instance_buffer.update(gl, &ground_instances);
        
        gl.draw_elements_instanced_with_i32(
            GL::TRIANGLES,
            self.ground_index_count,
            GL::UNSIGNED_SHORT,
            0,
            1,  // One instance for ground
        );
        
        // Draw pickable objects (BIG BOXES visible to all players)
        // Setup box mesh vertex attributes
        self.box_vbo.bind(gl);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
        gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
        gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
        gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
        self.box_ibo.bind(gl);
        
        if self.multiplayer_enabled {
            let my_id = self.network.player_id.clone();
            let mut box_instances: Vec<InstanceData> = self.network.pickables.iter()
                .filter_map(|obj| {
                    // Skip boxes held by US - we'll render them in front of camera
                    if let (Some(ref holder), Some(ref me)) = (&obj.held_by, &my_id) {
                        if holder == me {
                            return None;  // Skip our held box, render it separately
                        }
                    }
                    
                    // Orange/brown box color
                    let color = Color::rgb(0.9, 0.6, 0.2);
                    Some(InstanceData::new(
                        obj.position,
                        Vec3::new(2.0, 2.0, 2.0),  // 2x2x2 big box
                        color,
                    ))
                })
                .collect();
            
            // If WE are holding a box, render it in front of camera (visible first-person)
            if self.network.is_holding_object() {
                // Position box in front of camera, slightly lower and forward
                let held_pos = self.camera.position 
                    + self.camera.forward * 2.5  // In front
                    + Vec3::new(0.0, -0.8, 0.0); // Lower (below eye level)
                
                box_instances.push(InstanceData::new(
                    held_pos,
                    Vec3::new(1.2, 1.2, 1.2),  // Slightly smaller when held
                    Color::rgb(1.0, 0.7, 0.3),  // Brighter orange when held
                ));
            }
            
            if !box_instances.is_empty() {
                self.instance_buffer.update(gl, &box_instances);
                gl.draw_elements_instanced_with_i32(
                    GL::TRIANGLES,
                    self.box_index_count,
                    GL::UNSIGNED_SHORT,
                    0,
                    box_instances.len() as i32,
                );
            }
        } else {
            // In solo mode, show a local box for testing
            let box_instances = vec![InstanceData::new(
                Vec3::new(5.0, 1.0, 5.0),
                Vec3::new(2.0, 2.0, 2.0),
                Color::rgb(0.9, 0.6, 0.2),
            )];
            self.instance_buffer.update(gl, &box_instances);
            gl.draw_elements_instanced_with_i32(
                GL::TRIANGLES,
                self.box_index_count,
                GL::UNSIGNED_SHORT,
                0,
                1,
            );
        }

        if self.use_instancing {
            // INSTANCED RENDERING with LOD - Draw all particles in ONE call!
            
            // Gather instance data WITH LOD CULLING
            let mut instances = Vec::with_capacity(self.scene.entities.entities.len() + self.network.server_particles.len());
            let camera_pos = self.camera.position;
            
            // Add LOCAL particles (user-spawned)
            for &entity in &self.scene.entities.entities {
                if let (Some(transform), Some(color), Some(lifetime)) = (
                    self.scene.entities.transforms.get(entity),
                    self.scene.entities.colors.get(entity),
                    self.scene.entities.lifetimes.get(entity),
                ) {
                    // LOD culling - check if should render
                    let particle_radius = 0.1; // Small particle radius
                    let lod = self.lod_manager.calculate_lod(&transform.position, &camera_pos, particle_radius);
                    
                    if lod.should_render() {
                        // Adjust scale based on LOD
                        let mut final_scale = transform.scale * lod.get_mesh_quality();
                        
                        // SHRINK animation when lifetime is ending!
                        // Start shrinking in the last 0.5 seconds
                        let shrink_duration = 0.5;
                        if lifetime.remaining < shrink_duration {
                            // Smoothly shrink from 1.0 to 0.0 over the last 0.5 seconds
                            let shrink_factor = (lifetime.remaining / shrink_duration).max(0.0);
                            // Use easing for smoother animation (ease out)
                            let eased = shrink_factor * shrink_factor;
                            final_scale = final_scale * eased;
                        }
                        
                        // Also fade out the color alpha
                        let mut final_color = *color;
                        if lifetime.remaining < shrink_duration {
                            final_color.a *= (lifetime.remaining / shrink_duration).max(0.0);
                        }
                        
                        instances.push(InstanceData::new(
                            transform.position,
                            final_scale,
                            final_color,
                        ));
                    }
                }
            }
            
            // Add SERVER particles (rain, etc.) - synced from server
            for particle in &self.network.server_particles {
                let particle_radius = 0.1;
                let pos = particle.position;
                let lod = self.lod_manager.calculate_lod(&pos, &camera_pos, particle_radius);
                
                if lod.should_render() {
                    let base_scale = Vec3::splat(self.particle_size);
                    let final_scale = base_scale * lod.get_mesh_quality();
                    
                    // Shrink animation for server particles too
                    let shrink_duration = 0.5;
                    let mut scale = final_scale;
                    let mut alpha = 1.0f32;
                    if particle.lifetime < shrink_duration {
                        let shrink_factor = (particle.lifetime / shrink_duration).max(0.0);
                        let eased = shrink_factor * shrink_factor;
                        scale = scale * eased;
                        alpha = shrink_factor;
                    }
                    
                    instances.push(InstanceData::new(
                        pos,
                        scale,
                        Color::new(particle.color.r, particle.color.g, particle.color.b, alpha),
                    ));
                }
            }

            if !instances.is_empty() {
                // Enable blending for fade out effect
                gl.enable(GL::BLEND);
                gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
                
                // Bind particle mesh and setup vertex attributes
                self.particle_vbo.bind(gl);
                gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
                gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 12);
                gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 24);
                gl.vertex_attrib_pointer_with_i32(3, 4, GL::FLOAT, false, stride, 32);
                
                self.particle_ibo.bind(gl);
                self.instance_buffer.update(gl, &instances);

                // Draw ALL particles in ONE call using instancing!
                gl.draw_elements_instanced_with_i32(
                    GL::TRIANGLES,
                    self.particle_index_count,
                    GL::UNSIGNED_SHORT,
                    0,
                    instances.len() as i32,
                );
                
                gl.disable(GL::BLEND);
            }
        } else {
            // OLD METHOD - One draw call per particle (slow)
            let u_model = self.shader_program.get_uniform_location(gl, "u_model")
                .ok_or("Failed to get u_model location")?;

            self.particle_vbo.bind(gl);
            self.particle_ibo.bind(gl);

            for &entity in &self.scene.entities.entities {
                if let Some(transform) = self.scene.entities.transforms.get(entity) {
                    let model = transform.model_matrix();
                    gl.uniform_matrix4fv_with_f32_array(Some(&u_model), false, &model.as_array());
                    
                    gl.draw_elements_with_i32(
                        GL::TRIANGLES,
                        self.particle_index_count,
                        GL::UNSIGNED_SHORT,
                        0,
                    );
                }
            }
        }

        Ok(())
    }


    fn spawn_particle_burst(&mut self, count: u32) {
        use std::f32::consts::PI;
        
        // Cap at 500 particles per frame MAX for performance
        let capped_count = count.min(500);
        
        for _ in 0..capped_count {
            let angle = js_sys::Math::random() as f32 * PI * 2.0;
            let speed = (1.0 + js_sys::Math::random() as f32 * 3.0) * self.particle_speed;
            let height_speed = (2.0 + js_sys::Math::random() as f32 * 4.0) * self.particle_speed;

            let velocity = Velocity::new().with_linear(Vec3::new(
                angle.cos() * speed,
                height_speed,
                angle.sin() * speed,
            ));

            // Spawn at corner of the ground (like a fountain/spring)
            // Ground is 200x200 (ground_size * 2), so corner is at -ground_size, -ground_size
            let corner_pos = Vec3::new(-self.ground_size + 10.0, 0.0, -self.ground_size + 10.0);
            let mut transform = Transform::new().with_position(corner_pos);
            transform.scale = Vec3::splat(self.particle_size / 0.05); // Scale based on size

            let color = Color::new(
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
                js_sys::Math::random() as f32,
                1.0,
            );

            let lifetime = Lifetime::new(self.particle_lifetime * (0.5 + js_sys::Math::random() as f32 * 0.5));

            self.scene.entities.create_particle(transform, velocity, color, lifetime);
        }
    }
}

fn setup_mouse_events(_canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
    // These will be setup in the main start function with proper state access
    Ok(())
}

// Global state reference for control functions
static mut GAME_STATE_REF: Option<Rc<RefCell<GameState>>> = None;

#[wasm_bindgen]
pub fn set_spawn_rate(rate: u32) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                // Cap at 500 particles per frame MAX
                state.spawn_rate = rate.min(500);
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_particle_size(size: f32) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.particle_size = size;
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_particle_speed(speed: f32) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.particle_speed = speed;
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_particle_lifetime(lifetime: f32) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.particle_lifetime = lifetime;
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_instancing(enabled: bool) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.use_instancing = enabled;
                web_sys::console::log_1(&format!("🎨 GPU Instancing: {}", if enabled { "ON" } else { "OFF" }).into());
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_shadows(enabled: bool) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.shadows_enabled = enabled;
                web_sys::console::log_1(&format!("🌑 Shadows: {}", if enabled { "ON" } else { "OFF" }).into());
            }
        }
    }
}

#[wasm_bindgen]
pub fn set_pointer_locked(locked: bool) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.pointer_locked = locked;
            }
        }
    }
}

/// Get player labels data for rendering ranks above heads
/// Returns JSON: [{ id, x, y, rank, score, isMe, visible }]
#[wasm_bindgen]
pub fn get_player_labels() -> String {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(state) = state_ref.try_borrow() {
                let mut labels = Vec::new();
                let view_proj = state.camera.view_projection_matrix();
                let (width, height) = state.gl_context.get_viewport_size();
                
                // Collect all players with their scores
                // Use the SERVER's score data for everyone (including ourselves)
                let mut all_players: Vec<(String, Vec3, u32, bool)> = Vec::new();
                
                // First, find our own score from the server's player list
                let my_server_score = if let Some(ref my_id) = state.network.player_id {
                    state.network.remote_players.iter()
                        .find(|p| &p.id == my_id)
                        .map(|p| p.score)
                        .unwrap_or(0)
                } else {
                    0
                };
                
                // Add remote players (everyone except us - we use our local position)
                for player in &state.network.remote_players {
                    let is_me = if let Some(ref my_id) = state.network.player_id {
                        &player.id == my_id
                    } else {
                        false
                    };
                    
                    if is_me {
                        continue;  // Skip self - we'll add with local position below
                    }
                    
                    // Position above player's head (pill top)
                    let player_height = 5.0;
                    let base_scale_y = 2.5;
                    let base_height = 1.8 * base_scale_y;
                    let feet_y = player.position.y - player_height;
                    let head_y = feet_y + base_height + 1.0;  // Above head
                    
                    let world_pos = Vec3::new(player.position.x, head_y, player.position.z);
                    all_players.push((player.id.clone(), world_pos, player.score, false));
                }
                
                // Add local player with LOCAL position but SERVER score
                if let Some(ref my_id) = state.network.player_id {
                    let player_height = state.player_height;
                    let base_scale_y = 2.5;
                    let base_height = 1.8 * base_scale_y;
                    let feet_y = state.camera.position.y - player_height;
                    let head_y = feet_y + base_height + 1.0;
                    
                    let world_pos = Vec3::new(state.camera.position.x, head_y, state.camera.position.z);
                    // Use score from server, not local my_score
                    all_players.push((my_id.clone(), world_pos, my_server_score, true));
                }
                
                // Sort by score to calculate ranks
                all_players.sort_by(|a, b| b.2.cmp(&a.2));
                
                // Project to screen and create labels
                for (rank, (id, world_pos, score, is_me)) in all_players.iter().enumerate() {
                    // Project 3D to clip space
                    let clip = view_proj.transform_vec4(world_pos.x, world_pos.y, world_pos.z, 1.0);
                    
                    // Skip if behind camera
                    if clip.3 <= 0.0 {
                        continue;
                    }
                    
                    // Perspective divide to NDC
                    let ndc_x = clip.0 / clip.3;
                    let ndc_y = clip.1 / clip.3;
                    let ndc_z = clip.2 / clip.3;
                    
                    // Skip if outside frustum
                    if ndc_x < -1.0 || ndc_x > 1.0 || ndc_y < -1.0 || ndc_y > 1.0 || ndc_z < -1.0 || ndc_z > 1.0 {
                        continue;
                    }
                    
                    // Convert to screen coordinates
                    let screen_x = (ndc_x + 1.0) * 0.5 * width as f32;
                    let screen_y = (1.0 - ndc_y) * 0.5 * height as f32;  // Flip Y
                    
                    // Short ID for display
                    let short_id = if id.len() > 6 { &id[..6] } else { id };
                    
                    labels.push(format!(
                        r#"{{"id":"{}","x":{},"y":{},"rank":{},"score":{},"isMe":{}}}"#,
                        short_id, screen_x as i32, screen_y as i32, rank + 1, score, is_me
                    ));
                }
                
                return format!("[{}]", labels.join(","));
            }
        }
        "[]".to_string()
    }
}

/// Get leaderboard data
/// Returns JSON: { leaderboard: [...], myRank, myScore }
#[wasm_bindgen]
pub fn get_leaderboard() -> String {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(state) = state_ref.try_borrow() {
                let entries: Vec<String> = state.network.leaderboard.iter()
                    .map(|e| {
                        let short_id = if e.id.len() > 8 { &e.id[..8] } else { &e.id };
                        let is_me = state.network.player_id.as_ref() == Some(&e.id);
                        format!(r#"{{"id":"{}","rank":{},"score":{},"isMe":{}}}"#, 
                            short_id, e.rank, e.score, is_me)
                    })
                    .collect();
                
                return format!(
                    r#"{{"leaderboard":[{}],"myRank":{},"myScore":{}}}"#,
                    entries.join(","),
                    state.network.my_rank,
                    state.network.my_score
                );
            }
        }
        r#"{"leaderboard":[],"myRank":0,"myScore":0}"#.to_string()
    }
}

#[wasm_bindgen]
pub fn set_workers(enabled: bool) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.use_workers = enabled;
                web_sys::console::log_1(&format!("💻 Multi-Core Physics: {} (using {} cores)", 
                    if enabled { "ON" } else { "OFF" },
                    state.worker_pool.worker_count
                ).into());
            }
        }
    }
}

#[wasm_bindgen]
pub fn update_viewport(width: u32, height: u32, _is_mobile: bool) {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                // Update internal size tracking (canvas already resized by JS)
                state.gl_context.width = width;
                state.gl_context.height = height;
                
                // Update WebGL viewport
                state.gl_context.gl.viewport(0, 0, width as i32, height as i32);
                
                // Update camera aspect ratio
                state.camera.aspect = width as f32 / height as f32;
            }
        }
    }
}

#[wasm_bindgen]
pub fn clear_all_particles() {
    unsafe {
        if let Some(state_ref) = &GAME_STATE_REF {
            if let Ok(mut state) = state_ref.try_borrow_mut() {
                state.scene.clear();
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Better panic messages
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    web_sys::console::log_1(&"🎮 Initializing 3D Game Engine (WebGL)...".into());

    // Create game state
    let game_state = Rc::new(RefCell::new(GameState::new()?));
    
    // Store global reference for control functions
    unsafe {
        GAME_STATE_REF = Some(game_state.clone());
    }

    // Setup input events
    let window = web_sys::window().expect("no window");
    let canvas = game_state.borrow().gl_context.canvas.clone();
    
    // Keyboard events
    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                s.input.keyboard.press(event.code());
            }
        }) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                s.input.keyboard.release(event.code());
            }
        }) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mouse events - use movement_x/y for pointer lock support
    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                // Use movement delta (works with pointer lock!)
                s.input.mouse.update_delta(event.movement_x() as f32, event.movement_y() as f32);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                if event.button() == 0 {
                    s.input.mouse.press_left();
                } else if event.button() == 2 {
                    s.input.mouse.press_right();
                }
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                if event.button() == 0 {
                    s.input.mouse.release_left();
                } else if event.button() == 2 {
                    s.input.mouse.release_right();
                }
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            if let Ok(mut s) = state.try_borrow_mut() {
                s.input.mouse.update_wheel(event.delta_y() as f32);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Disable right-click context menu on canvas
    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

        web_sys::console::log_1(&"✅ Engine initialized! Starting 3D rendering...".into());
        
        // Connect to multiplayer server
        // For dev: backend runs in same Docker network
        // For prod: set to your Fly.io backend URL
        let server_url = "ws://localhost:9001"; // Works for both local dev and Docker!
        
        if let Ok(mut state) = game_state.try_borrow_mut() {
            match state.network.connect(server_url) {
                Ok(_) => {
                    state.multiplayer_enabled = true;
                    web_sys::console::log_1(&"🌐 Multiplayer enabled!".into());
                }
                Err(_) => {
                    web_sys::console::log_1(&"⚠️ Multiplayer server not available (running solo mode)".into());
                }
            }
        }

    // Start game loop with stats tracking
    {
        let game_state = game_state.clone();
        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let performance = window.performance().expect("no performance");
        
        // Get stat elements
        let fps_element = document.get_element_by_id("fps-counter");
        let frame_time_element = document.get_element_by_id("frame-time");
        let particle_count_element = document.get_element_by_id("particle-count");
        let visible_count_element = document.get_element_by_id("visible-count");
        let culled_count_element = document.get_element_by_id("culled-count");
        let draw_calls_element = document.get_element_by_id("draw-calls");
        let memory_element = document.get_element_by_id("memory-usage");
        let player_count_element = document.get_element_by_id("player-count");
        
        run_game_loop(move |time| {
            if let Ok(mut state) = game_state.try_borrow_mut() {
                let frame_start = performance.now();
                
                state.engine.update_time(time);
                
                // Clone input for update (to avoid borrow issues)
                let input = state.input.clone();
                state.update(&input);
                
                // Clear input frame state
                state.input.clear_frame_state();
                
                let render_result = state.render();
                let frame_end = performance.now();
                let frame_time = frame_end - frame_start;
                
                // Update stats (every frame for smooth updates)
                let particle_count = state.scene.entities.entity_count();
                
                if let Some(fps_elem) = fps_element.as_ref() {
                    fps_elem.set_text_content(Some(&format!("FPS: {:.0}", state.engine.fps)));
                }
                
                if let Some(ft_elem) = frame_time_element.as_ref() {
                    let color = if frame_time < 16.0 { "#0f0" } else if frame_time < 33.0 { "#ff0" } else { "#f00" };
                    ft_elem.set_inner_html(&format!("Frame: <span style='color:{}'>{:.2}ms</span>", color, frame_time));
                }
                
                let lod_stats = &state.lod_manager.stats;
                
                if let Some(pc_elem) = particle_count_element.as_ref() {
                    pc_elem.set_text_content(Some(&format!("Total: {}", particle_count)));
                }
                
                if let Some(vc_elem) = visible_count_element.as_ref() {
                    let pct = if particle_count > 0 { 
                        (lod_stats.visible_objects as f32 / particle_count as f32 * 100.0) as u32 
                    } else { 
                        0 
                    };
                    vc_elem.set_inner_html(&format!("Visible: <span style='color:#0f0'>{}</span> ({}%)", lod_stats.visible_objects, pct));
                }
                
                if let Some(cc_elem) = culled_count_element.as_ref() {
                    let total_culled = lod_stats.culled_by_frustum + lod_stats.culled_by_distance;
                    cc_elem.set_inner_html(&format!("Culled: <span style='color:#f80'>{}</span> (F:{} D:{})", 
                        total_culled, 
                        lod_stats.culled_by_frustum,
                        lod_stats.culled_by_distance
                    ));
                }
                
                if let Some(dc_elem) = draw_calls_element.as_ref() {
                    if state.use_instancing {
                        // Instanced: sun + player + ground + clouds + all particles = 5 total
                        let actual_draws = 5;
                        dc_elem.set_inner_html(&format!("Draw Calls: <span style='color:#0f0'>{}</span> (instanced ✓)", actual_draws));
                    } else {
                        // Non-instanced: sun + player + ground + clouds + 1 per particle
                        let actual_draws = 4 + particle_count;
                        dc_elem.set_inner_html(&format!("Draw Calls: <span style='color:#f00'>{}</span> (not instanced ✗)", actual_draws));
                    }
                }
                
                // Update player count
                if let Some(pc_elem) = player_count_element.as_ref() {
                    let count = state.network.player_count;
                    if count > 1 {
                        pc_elem.set_inner_html(&format!("👥 Players: <span style='color:#0f0'>{}</span>", count));
                    } else {
                        pc_elem.set_inner_html("👥 Players: <span style='color:#ff0'>1</span> (solo)");
                    }
                }
                
                // Get memory usage (if available)
                if let Some(mem_elem) = memory_element.as_ref() {
                    if let Ok(memory) = js_sys::Reflect::get(&performance, &"memory".into()) {
                        if let Some(used_heap) = js_sys::Reflect::get(&memory, &"usedJSHeapSize".into())
                            .ok()
                            .and_then(|v| v.as_f64()) 
                        {
                            let mb = used_heap / 1024.0 / 1024.0;
                            mem_elem.set_text_content(Some(&format!("Memory: {:.1}MB", mb)));
                        } else {
                            mem_elem.set_text_content(Some("Memory: --"));
                        }
                    } else {
                        mem_elem.set_text_content(Some("Memory: --"));
                    }
                }
                
                if let Err(e) = render_result {
                    web_sys::console::error_1(&format!("Render error: {:?}", e).into());
                }
            }
        });
    }

    Ok(())
}
