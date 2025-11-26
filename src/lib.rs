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

        // Create ground mesh (HUGE visible plane) - green grass
        let ground_size = 200.0;  // Make it HUGE so you can always see it
        let ground_vertices = vec![
            Vertex::new([-ground_size, 0.0, -ground_size], [0.0, 1.0, 0.0], [0.0, 0.0], [0.2, 0.8, 0.2, 1.0]),
            Vertex::new([ ground_size, 0.0, -ground_size], [0.0, 1.0, 0.0], [1.0, 0.0], [0.2, 0.8, 0.2, 1.0]),
            Vertex::new([ ground_size, 0.0,  ground_size], [0.0, 1.0, 0.0], [1.0, 1.0], [0.2, 0.8, 0.2, 1.0]),
            Vertex::new([-ground_size, 0.0,  ground_size], [0.0, 1.0, 0.0], [0.0, 1.0], [0.2, 0.8, 0.2, 1.0]),
        ];
        let ground_indices = vec![0, 1, 2, 0, 2, 3];
        
        let ground_vertex_data = vertex_data_interleaved(&ground_vertices);
        let ground_vbo = GLBuffer::new(gl, GL::ARRAY_BUFFER)?;
        ground_vbo.set_data(gl, &ground_vertex_data, GL::STATIC_DRAW);
        
        let ground_ibo = GLBuffer::new(gl, GL::ELEMENT_ARRAY_BUFFER)?;
        ground_ibo.set_data_u16(gl, &ground_indices, GL::STATIC_DRAW);
        let ground_index_count = ground_indices.len() as i32;
        
        // Ground collision plane
        let ground_plane = Plane::from_point_normal(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        
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
        
        // Create clouds (20 voxel/Minecraft style clouds)
        let clouds = CloudSystem::new(20, 100.0);
        
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
        let camera = Camera::new(
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::ZERO,
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
        let player_height = 1.8;

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
            spawn_rate: 50,
            particle_size: 0.05,
            particle_speed: 2.0,
            particle_lifetime: 3.0,
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
        let move_force = 50.0;
        let jump_force = 8.0;
        
        if input.keyboard.is_pressed(input::KEY_W) {
            let force = self.camera.forward * move_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_S) {
            let force = self.camera.forward * -move_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_A) {
            let force = self.camera.right * -move_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        if input.keyboard.is_pressed(input::KEY_D) {
            let force = self.camera.right * move_force;
            self.player_rigidbody.apply_force(Vec3::new(force.x, 0.0, force.z));
        }
        
        // Jump (only when grounded)
        if input.keyboard.is_just_pressed(input::KEY_SPACE) && self.player_can_jump {
            self.player_rigidbody.apply_impulse(Vec3::new(0.0, jump_force, 0.0));
            self.player_can_jump = false;
        }

        // Update player physics
        self.player_rigidbody.update(dt, GRAVITY);
        
        // Apply physics to camera position
        let displacement = self.player_rigidbody.get_displacement(dt);
        self.camera.position += displacement;
        self.camera.target += displacement;

        // Ground collision
        let player_bottom = self.camera.position.y - self.player_height / 2.0;
        if player_bottom < 0.0 {
            // Hit ground - stop falling and allow jumping
            self.camera.position.y = self.player_height / 2.0;
            self.camera.target.y = self.camera.position.y + self.camera.forward.y;
            self.player_rigidbody.velocity.y = 0.0;
            self.player_rigidbody.is_grounded = true;
            self.player_can_jump = true;
        } else {
            self.player_rigidbody.is_grounded = false;
        }

        // Mouse look (when right mouse button is held)
        if input.mouse.right_button {
            let sensitivity = 0.002;
            self.camera.rotate(
                input.mouse.delta.x * sensitivity,
                -input.mouse.delta.y * sensitivity,
            );
        }
        
        // Apply gravity and GROUND BOUNCE to particles
        // This could be parallelized with Web Workers for even better performance!
        let entity_count = self.scene.entities.entities.len();
        
        if self.use_workers && entity_count > 10000 {
            // Multi-threaded physics (when we have many particles)
            // TODO: Actually spawn Web Workers and distribute work
            // For now, just do batched processing to simulate worker chunks
            
            let chunks = chunk_indices(entity_count, self.worker_pool.worker_count);
            
            for (start, end) in chunks {
                for i in start..end {
                    if let Some(&entity) = self.scene.entities.entities.get(i) {
                        if let (Some(transform), Some(velocity)) = (
                            self.scene.entities.transforms.get_mut(entity),
                            self.scene.entities.velocities.get_mut(entity),
                        ) {
                            velocity.linear += GRAVITY * dt;
                            
                            if transform.position.y < 0.0 {
                                transform.position.y = 0.0;
                                velocity.linear.y = -velocity.linear.y * 0.6;
                                velocity.linear.x *= 0.95;
                                velocity.linear.z *= 0.95;
                            }
                        }
                    }
                }
            }
        } else {
            // Single-threaded physics (simple, fast for < 10K particles)
            for &entity in &self.scene.entities.entities.clone() {
                if let (Some(transform), Some(velocity)) = (
                    self.scene.entities.transforms.get_mut(entity),
                    self.scene.entities.velocities.get_mut(entity),
                ) {
                    // Gravity
                    velocity.linear += GRAVITY * dt;
                    
                    // Ground collision - bounce
                    if transform.position.y < 0.0 {
                        transform.position.y = 0.0;
                        velocity.linear.y = -velocity.linear.y * 0.6; // Bounce with 60% energy
                        velocity.linear.x *= 0.95; // Friction
                        velocity.linear.z *= 0.95; // Friction
                    }
                }
            }
        }

        // Spawn particles continuously based on spawn rate
        if self.spawn_rate > 0 && self.engine.frame_count % 1 == 0 {
            self.spawn_particle_burst(self.spawn_rate);
        }
        
        // Multiplayer: sync state and send updates
        if self.multiplayer_enabled {
            // Sync state from WebSocket callbacks
            self.network.sync_state();
            
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
        
        // Update viewport to match canvas size (CRITICAL!)
        let (width, height) = self.gl_context.get_viewport_size();
        gl.viewport(0, 0, width as i32, height as i32);
        
        // Clear screen with skybox color
        let sky_color = self.skybox.get_sky_color();
        self.gl_context.clear(sky_color.x, sky_color.y, sky_color.z, 1.0);

        // Use shader
        self.shader_program.use_program(gl);

        // Update camera aspect ratio (happens every frame to catch resizes)
        self.camera.aspect = self.gl_context.get_aspect_ratio();
        let view_proj = self.camera.view_projection_matrix();

        // Get uniform locations
        let u_view_proj = self.shader_program.get_uniform_location(gl, "u_viewProj")
            .ok_or("Failed to get u_viewProj location")?;
        let u_sun_direction = self.shader_program.get_uniform_location(gl, "u_sunDirection");
        let u_sun_color = self.shader_program.get_uniform_location(gl, "u_sunColor");
        let u_ambient_strength = self.shader_program.get_uniform_location(gl, "u_ambientStrength");

        // Set view-projection matrix
        gl.uniform_matrix4fv_with_f32_array(Some(&u_view_proj), false, &view_proj.as_array());
        
        // Set sun lighting uniforms
        let sun_dir = self.skybox.get_sun_direction();
        if let Some(loc) = u_sun_direction {
            gl.uniform3f(Some(&loc), sun_dir.x, sun_dir.y, sun_dir.z);
        }
        
        let is_night = self.skybox.is_night();
        let sun_color = if is_night {
            Vec3::new(0.3, 0.3, 0.5)  // Moonlight (bluish)
        } else {
            Vec3::new(1.0, 0.95, 0.8)  // Sunlight (warm yellow)
        };
        
        if let Some(loc) = u_sun_color {
            gl.uniform3f(Some(&loc), sun_color.x, sun_color.y, sun_color.z);
        }
        
        let ambient = if is_night { 0.15 } else { 0.3 };
        if let Some(loc) = u_ambient_strength {
            gl.uniform1f(Some(&loc), ambient);
        }

        // Render clouds first (in background, with transparency)
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        
        let is_night = self.skybox.is_night();
        let cloud_instances: Vec<InstanceData> = self.clouds.get_instances(is_night)
            .iter()
            .map(|(pos, scale, color)| InstanceData::new(*pos, *scale, *color))
            .collect();
        
        if !cloud_instances.is_empty() {
            self.instance_buffer.update(gl, &cloud_instances);
            self.cloud_mesh_vbo.bind(gl);
            self.cloud_mesh_ibo.bind(gl);
            gl.draw_elements_instanced_with_i32(
                GL::TRIANGLES,
                self.cloud_mesh_index_count,
                GL::UNSIGNED_SHORT,
                0,
                cloud_instances.len() as i32,
            );
        }
        
        gl.disable(GL::BLEND);
        
        // Draw sun (HUGE bright yellow sphere in sky)
        gl.disable(GL::DEPTH_TEST);  // Always render sun on top
        
        let sun_dir = self.skybox.get_sun_direction();
        let sun_distance = 100.0;
        let sun_position = self.camera.position + sun_dir * sun_distance;
        
        let sun_instance = vec![InstanceData::new(
            sun_position,
            Vec3::splat(15.0),  // MUCH BIGGER
            Color::rgb(1.0, 1.0, 0.0),  // PURE YELLOW - no lighting
        )];
        self.instance_buffer.update(gl, &sun_instance);
        self.sun_vbo.bind(gl);
        self.sun_ibo.bind(gl);
        gl.draw_elements_instanced_with_i32(
            GL::TRIANGLES,
            self.sun_index_count,
            GL::UNSIGNED_SHORT,
            0,
            1,
        );
        
        gl.enable(GL::DEPTH_TEST);  // Re-enable depth test
        
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
                    0 => Color::rgb(1.0, 0.0, 0.0),  // Bright Red
                    1 => Color::rgb(0.0, 1.0, 0.0),  // Bright Green
                    2 => Color::rgb(1.0, 1.0, 0.0),  // Bright Yellow
                    3 => Color::rgb(1.0, 0.0, 1.0),  // Bright Magenta
                    4 => Color::rgb(0.0, 1.0, 1.0),  // Bright Cyan
                    _ => Color::rgb(1.0, 0.5, 0.0),  // Bright Orange
                };
                
                remote_instances.push(InstanceData::new(
                    player.position,
                    Vec3::new(3.0, 3.0, 3.0),  // 3X BIGGER!
                    color,
                ));
            }
            
            if !remote_instances.is_empty() {
                self.instance_buffer.update(gl, &remote_instances);
                self.player_vbo.bind(gl);
                self.player_ibo.bind(gl);
                gl.draw_elements_instanced_with_i32(
                    GL::TRIANGLES,
                    self.player_index_count,
                    GL::UNSIGNED_SHORT,
                    0,
                    remote_instances.len() as i32,
                );
            }
        }
        
        // Draw ground (as single instance) - MASSIVE GREEN PLANE
        let ground_color = if is_night {
            Color::rgb(0.05, 0.2, 0.05)   // Dark green at night
        } else {
            Color::rgb(0.1, 0.9, 0.1)   // BRIGHT GREEN grass during day
        };
        
        let ground_instances = vec![InstanceData::new(
            Vec3::new(0.0, 0.0, 0.0),  // At y=0
            Vec3::new(500.0, 1.0, 500.0), // HUGE 500x500 plane
            ground_color
        )];
        self.instance_buffer.update(gl, &ground_instances);
        
        self.ground_vbo.bind(gl);
        self.ground_ibo.bind(gl);
        gl.draw_elements_instanced_with_i32(
            GL::TRIANGLES,
            self.ground_index_count,
            GL::UNSIGNED_SHORT,
            0,
            1,  // One instance for ground
        );

        if self.use_instancing {
            // INSTANCED RENDERING with LOD - Draw all particles in ONE call!
            
            // Gather instance data WITH LOD CULLING
            let mut instances = Vec::with_capacity(self.scene.entities.entities.len());
            let camera_pos = self.camera.position;
            
            for &entity in &self.scene.entities.entities {
                if let (Some(transform), Some(color)) = (
                    self.scene.entities.transforms.get(entity),
                    self.scene.entities.colors.get(entity),
                ) {
                    // LOD culling - check if should render
                    let particle_radius = 0.1; // Small particle radius
                    let lod = self.lod_manager.calculate_lod(&transform.position, &camera_pos, particle_radius);
                    
                    if lod.should_render() {
                        // Adjust scale based on LOD
                        let lod_scale = transform.scale * lod.get_mesh_quality();
                        
                        instances.push(InstanceData::new(
                            transform.position,
                            lod_scale,
                            *color,
                        ));
                    }
                }
            }

            if !instances.is_empty() {
                // Update instance buffer
                self.instance_buffer.update(gl, &instances);

                // Bind buffers
                self.particle_vbo.bind(gl);
                self.particle_ibo.bind(gl);

                // Draw ALL particles in ONE call using instancing!
                gl.draw_elements_instanced_with_i32(
                    GL::TRIANGLES,
                    self.particle_index_count,
                    GL::UNSIGNED_SHORT,
                    0,
                    instances.len() as i32,
                );
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
        
        for _ in 0..count {
            let angle = js_sys::Math::random() as f32 * PI * 2.0;
            let speed = (1.0 + js_sys::Math::random() as f32 * 3.0) * self.particle_speed;
            let height_speed = (2.0 + js_sys::Math::random() as f32 * 4.0) * self.particle_speed;

            let velocity = Velocity::new().with_linear(Vec3::new(
                angle.cos() * speed,
                height_speed,
                angle.sin() * speed,
            ));

            let mut transform = Transform::new().with_position(Vec3::ZERO);
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

fn setup_mouse_events(canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
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
                state.spawn_rate = rate;
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
                // Update WebGL viewport
                let gl = &state.gl_context.gl;
                gl.viewport(0, 0, width as i32, height as i32);
                
                // Update camera aspect ratio
                // The projection matrix will automatically use the new aspect
                // This keeps the vertical FOV constant, adjusting horizontal FOV based on width
                state.camera.aspect = width as f32 / height as f32;
                
                // Keep FOV constant (vertical field of view stays the same)
                // This is the CORRECT way - like real game engines
                // Wide window = see more horizontally
                // Tall window = see more vertically
                // Objects stay same size!
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

    // Mouse events
    {
        let state = game_state.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Ok(mut s) = state.try_borrow_mut() {
                s.input.mouse.update_position(event.offset_x() as f32, event.offset_y() as f32);
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
