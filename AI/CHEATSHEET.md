# 🚀 Quick Reference Cheat Sheet

## 📁 Project Structure
```
src/
├── math/        → Vec2, Vec3, Vec4, Mat4, Quat
├── core/        → Engine loop, memory allocators
├── renderer/    → WebGPU, buffers, shaders, camera
├── ecs/         → Entity-Component System
├── input/       → Keyboard, mouse
├── assets/      → Mesh generation
└── scene/       → Scene management
```

---

## 🧮 Math Quick Reference

### Vectors
```rust
// Creation
let v = Vec3::new(1.0, 2.0, 3.0);
let v = Vec3::ZERO;  // (0, 0, 0)
let v = Vec3::ONE;   // (1, 1, 1)
let v = Vec3::splat(5.0);  // (5, 5, 5)

// Operations
v1 + v2          // Add
v1 - v2          // Subtract
v * 2.0          // Scale
v.dot(&v2)       // Dot product (similarity)
v.cross(&v2)     // Cross product (perpendicular)
v.normalize()    // Make length 1
v.length()       // Get length
v.lerp(&v2, 0.5) // Interpolate 50%
```

### Matrices
```rust
// Transformations
Mat4::translate(x, y, z)
Mat4::scale(x, y, z)
Mat4::rotate_x(angle)
Mat4::rotate_y(angle)
Mat4::rotate_z(angle)

// Projections
Mat4::perspective(fov, aspect, near, far)
Mat4::orthographic(left, right, bottom, top, near, far)
Mat4::look_at(&eye, &target, &up)

// Combine (order matters!)
let model = translation.mul(&rotation).mul(&scale);
```

### Quaternions
```rust
// Creation
Quat::from_axis_angle(&Vec3::Y, angle)
Quat::from_euler(pitch, yaw, roll)

// Operations
q1.mul(&q2)        // Combine rotations
q.slerp(&q2, t)    // Smooth interpolation
q.to_mat4()        // Convert to matrix
```

---

## 🧠 Memory Management

### Arena (Fast temporary allocations)
```rust
let mut arena = Arena::new(1024 * 1024);  // 1MB
let ptr = arena.alloc::<T>(count)?;
arena.reset();  // Free all at once
```

### Pool (Fixed-size objects)
```rust
let mut pool = Pool::<Entity>::new(10000);
let id = pool.alloc(entity)?;
let entity = pool.get(id);
pool.free(id);
```

### Generational Pool (Safe handles)
```rust
let mut pool = GenerationalPool::<T>::new(1000);
let handle = pool.alloc(value)?;
let value = pool.get(handle);  // Returns None if freed
pool.free(handle);
```

---

## 🎨 Rendering

### GPU Context
```rust
let gpu = GpuContext::new(canvas).await?;
gpu.resize(width, height);
let texture = gpu.get_current_texture();
```

### Buffers
```rust
// Vertex buffer
let vbo = Buffer::vertex(&device, as_bytes(&vertices))?;

// Index buffer
let ibo = Buffer::index(&device, as_bytes(&indices))?;

// Uniform buffer
let ubo = Buffer::uniform(&device, size)?;
ubo.write(&queue, data, offset);
```

### Shaders
```rust
let shader = Shader::new(&device, WGSL_CODE)?;
```

### Pipeline
```rust
let pipeline = RenderPipeline::new(
    &device,
    &vertex_shader,
    &fragment_shader,
    surface_format,
    &bind_group_layouts,
)?;
```

### Camera
```rust
let mut camera = Camera::new(
    Vec3::new(0.0, 5.0, 10.0),  // Position
    Vec3::ZERO,                  // Target
    deg_to_rad(45.0),           // FOV
    aspect_ratio,
);

camera.orbit(delta_yaw, delta_pitch);
camera.zoom(delta);
let vp = camera.view_projection_matrix();
```

---

## 🎯 ECS (Entity Component System)

### Components
```rust
// Transform
let t = Transform::new()
    .with_position(Vec3::new(1.0, 2.0, 3.0))
    .with_rotation(Quat::from_axis_angle(&Vec3::Y, angle))
    .with_scale(Vec3::ONE);

// Velocity
let v = Velocity::new()
    .with_linear(Vec3::new(1.0, 0.0, 0.0));

// Color
let c = Color::rgb(1.0, 0.0, 0.0);  // Red
let c = Color::RED;                 // Preset

// Lifetime
let l = Lifetime::new(5.0);  // 5 seconds
if l.is_alive() { /* ... */ }
```

### Entity Manager
```rust
let mut entities = EntityManager::new(100000);

// Create entity
let entity = entities.create_entity(transform)?;

// Create particle (with all components)
let entity = entities.create_particle(
    transform,
    velocity,
    color,
    lifetime,
)?;

// Destroy entity
entities.destroy_entity(entity);

// Update systems
entities.update_physics(delta_time);
entities.update_lifetimes(delta_time);
```

---

## 🎮 Input

### Keyboard
```rust
if input.keyboard.is_pressed(KEY_W) { /* ... */ }
if input.keyboard.is_just_pressed(KEY_SPACE) { /* ... */ }

// Key codes
KEY_W, KEY_A, KEY_S, KEY_D
KEY_SPACE, KEY_SHIFT, KEY_CTRL
ARROW_UP, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT
```

### Mouse
```rust
let pos = input.mouse.position;      // Vec2
let delta = input.mouse.delta;       // Vec2
let wheel = input.mouse.wheel_delta; // f32

if input.mouse.left_button { /* ... */ }
if input.mouse.left_just_pressed { /* ... */ }
```

---

## 📦 Assets

### Meshes
```rust
let cube = Mesh::cube(1.0);
let sphere = Mesh::sphere(1.0, 32, 32);  // radius, stacks, slices
let quad = Mesh::quad(1.0);

let vbo = Buffer::vertex(&device, as_bytes(&mesh.vertices))?;
let ibo = Buffer::index(&device, as_bytes(&mesh.indices))?;
```

---

## 🌍 Scene

### Scene Management
```rust
let mut scene = Scene::new(1_000_000);

// Update
scene.update(&engine);

// Clear
scene.clear();
```

---

## ⚙️ Engine Loop

### Engine
```rust
let mut engine = Engine::new();

// In game loop
engine.update_time(current_time);
println!("FPS: {}", engine.fps);
println!("DT: {}", engine.delta_time);
```

### Game Loop
```rust
run_game_loop(|time| {
    engine.update_time(time);
    update();
    render();
});
```

---

## 🔧 Common Patterns

### Spawn Particle
```rust
let transform = Transform::new()
    .with_position(Vec3::ZERO);

let velocity = Velocity::new()
    .with_linear(Vec3::new(
        angle.cos() * speed,
        height_speed,
        angle.sin() * speed,
    ));

let color = Color::rgb(
    rand() as f32,
    rand() as f32,
    rand() as f32,
);

let lifetime = Lifetime::new(3.0);

scene.entities.create_particle(
    transform,
    velocity,
    color,
    lifetime,
);
```

### Update Uniforms
```rust
let view_proj = camera.view_projection_matrix();
let model = transform.model_matrix();

let mut data = Vec::new();
data.extend_from_slice(&view_proj.as_array());
data.extend_from_slice(&model.as_array());
data.push(engine.total_time);

uniform_buffer.write(&queue, as_bytes(&data), 0);
```

### Render Pass
```rust
let encoder = device.create_command_encoder();
let render_pass = encoder.begin_render_pass(&descriptor);

render_pass.set_pipeline(&pipeline.pipeline);
render_pass.set_bind_group(0, Some(&bind_group));
render_pass.set_vertex_buffer(0, &vertex_buffer.buffer);
render_pass.set_index_buffer(&index_buffer.buffer, "uint16");

render_pass.draw_indexed(index_count, 1, 0, 0, 0);
render_pass.end();

let command_buffer = encoder.finish();
queue.submit(&[command_buffer]);
```

---

## 🐛 Debugging

### Console Logging
```rust
web_sys::console::log_1(&"Message".into());
web_sys::console::log_1(&format!("FPS: {}", fps).into());
```

### Performance
```rust
let start = performance.now();
// ... code ...
let elapsed = performance.now() - start;
console::log(&format!("Took: {}ms", elapsed));
```

---

## 🚀 Deployment

### Local Dev
```bash
./run-dev.sh
# Open http://localhost:8080
```

### Production
```bash
./deploy-prod.sh
# Deploys to Fly.io
```

---

## 📊 Performance Tips

1. **Batch operations** - Update all physics at once
2. **Use instancing** - Draw many objects in one call
3. **Minimize uniform updates** - Only when changed
4. **Profile first** - Don't optimize blindly
5. **GPU over CPU** - Let GPU do heavy lifting

---

## 🎓 Learning Path

1. ✅ Understand the code structure
2. ✅ Modify particle colors
3. ✅ Change spawn patterns
4. ✅ Add gravity
5. ⬜ Implement instanced rendering
6. ⬜ Add compute shaders
7. ⬜ Build a game!

---

Quick reference for the **3D Game Engine** built from scratch!

