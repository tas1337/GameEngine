# 🏗️ Engine Architecture

## Overview

This 3D game engine is built **100% from scratch** using Rust and raw WebGPU bindings. Every component is custom-built for maximum learning and understanding.

---

## 🧮 **1. Math Library** (`src/math/`)

### Purpose
Foundation for all 3D transformations and calculations.

### Components

#### `vec2.rs` - 2D Vectors
- Position, velocity, UV coordinates
- Operations: add, subtract, multiply, divide, dot product, normalize
- Used for: Mouse input, texture coordinates

#### `vec3.rs` - 3D Vectors
- 3D positions, normals, velocities
- Operations: all Vec2 ops + cross product, reflect
- Used for: Entity positions, camera direction, particle velocities

#### `vec4.rs` - 4D Vectors
- Colors (RGBA), homogeneous coordinates
- Operations: all Vec2/Vec3 ops
- Used for: Colors, matrix transformations

#### `mat4.rs` - 4x4 Matrices
- **Column-major** layout (WebGPU standard)
- Constructors: translate, scale, rotate (X/Y/Z)
- Projections: perspective, orthographic
- Camera: look-at matrix
- Operations: matrix multiplication, vector transformation

#### `quat.rs` - Quaternions
- Smooth rotations without gimbal lock
- Constructors: from axis-angle, from Euler angles
- Operations: multiplication, SLERP interpolation
- Conversion: to Mat4

### Design Philosophy
- **No external dependencies** - everything handwritten
- **SIMD-friendly** - #[repr(C)] for GPU compatibility
- **Performance** - inline everything, zero-cost abstractions

---

## 🧠 **2. Core Systems** (`src/core/`)

### `memory.rs` - Custom Allocators

#### Arena Allocator
```rust
Arena::new(capacity) -> Self
Arena::alloc<T>(count) -> Option<NonNull<T>>
Arena::reset()
```
- **Use case**: Temporary per-frame allocations
- **Benefits**: Fast bulk allocation, no fragmentation
- **Trade-off**: Can't free individual objects

#### Pool Allocator
```rust
Pool<T>::new(capacity) -> Self
Pool::alloc(value) -> Option<usize>
Pool::free(index)
```
- **Use case**: Fixed-size objects (entities, components)
- **Benefits**: O(1) allocation/deallocation
- **Implementation**: Free list + slot array

#### Generational Pool
```rust
GenerationalPool<T>::new(capacity) -> Self
GenerationalPool::alloc(value) -> Option<Handle>
GenerationalPool::free(handle) -> bool
```
- **Use case**: Safe handles that detect stale references
- **Benefits**: Prevent use-after-free bugs
- **Implementation**: Generation counter per slot

### `engine.rs` - Main Loop

#### Engine State
- `delta_time`: Time since last frame (seconds)
- `total_time`: Cumulative running time
- `frame_count`: Total frames rendered
- `fps`: Frames per second

#### Game Loop
```rust
run_game_loop(callback)
```
- Uses `requestAnimationFrame` for 60 FPS
- Calculates delta time automatically
- Updates FPS counter every second

---

## 🎨 **3. Renderer** (`src/renderer/`)

### `gpu_context.rs` - WebGPU Setup

#### Initialization
1. Get GPU instance from navigator
2. Request adapter (GPU hardware access)
3. Request device (GPU logical interface)
4. Configure canvas surface
5. Set up texture format (BGRA8)

#### Responsibilities
- Device and queue management
- Surface configuration
- Window resizing
- Texture retrieval

### `buffer.rs` - GPU Memory

#### Buffer Types
- **Vertex Buffer**: Geometry data
- **Index Buffer**: Triangle indices
- **Uniform Buffer**: Shader constants (matrices, time)

#### Vertex Format
```rust
struct Vertex {
    position: [f32; 3],  // 12 bytes
    normal: [f32; 3],    // 12 bytes
    uv: [f32; 2],        // 8 bytes
    color: [f32; 4],     // 16 bytes
}                        // Total: 48 bytes
```

#### Memory Layout
- **Column-major matrices** for WebGPU
- **Aligned to 16 bytes** for GPU efficiency
- **Zero-copy uploads** where possible

### `shader.rs` - WGSL Shaders

#### Vertex Shader
```wgsl
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    world_pos = model * position
    clip_pos = view_proj * world_pos
    return VertexOutput
}
```

#### Fragment Shader
```wgsl
@fragment
fn fs_main(in: FragmentInput) -> @location(0) vec4<f32> {
    lighting = calculate_lighting(normal)
    return color * lighting
}
```

### `pipeline.rs` - Render Pipeline

#### Setup Process
1. Define vertex buffer layout (attributes, stride)
2. Create vertex state (shader, entry point, buffers)
3. Create fragment state (shader, targets)
4. Set primitive state (topology, culling)
5. Create pipeline layout (bind groups)
6. Compile pipeline

### `camera.rs` - View System

#### Camera Components
- **Position**: Eye location in world space
- **Target**: Look-at point
- **Up**: Up vector (usually Y-axis)
- **FOV**: Field of view (radians)
- **Aspect**: Width / height ratio
- **Near/Far**: Clipping planes

#### Operations
- **View Matrix**: World → Camera space
- **Projection Matrix**: Camera → Clip space
- **Orbit**: Spherical coordinate rotation
- **Zoom**: Distance adjustment

---

## 🎯 **4. ECS (Entity Component System)** (`src/ecs/`)

### `component.rs` - Component Definitions

#### Transform
- Position, rotation (quaternion), scale
- `model_matrix()` → converts to Mat4

#### Velocity
- Linear velocity (Vec3)
- Angular velocity (Vec3)

#### Color
- RGBA (f32 x 4)
- Presets: WHITE, RED, GREEN, BLUE

#### Lifetime
- Remaining time (seconds)
- Total time
- `is_alive()` → bool

### `entity.rs` - Entity Manager

#### Architecture
```
Entity = Handle (index + generation)
EntityManager {
    transforms: GenerationalPool<Transform>
    velocities: GenerationalPool<Velocity>
    colors: GenerationalPool<Color>
    lifetimes: GenerationalPool<Lifetime>
    entities: Vec<Entity>
}
```

#### Operations
- `create_entity(transform)` → Entity
- `create_particle(all components)` → Entity
- `destroy_entity(entity)`
- `update_physics(delta_time)`
- `update_lifetimes(delta_time)`

#### Data-Oriented Design
- Components stored in separate arrays
- Cache-friendly iteration
- Easy to add new component types

---

## 🎮 **5. Input System** (`src/input/`)

### `keyboard.rs` - Keyboard Input

#### State Tracking
- `pressed`: Currently held keys
- `just_pressed`: Keys pressed this frame
- `just_released`: Keys released this frame

#### Methods
- `is_pressed(key)` → bool
- `is_just_pressed(key)` → bool
- `is_just_released(key)` → bool
- `clear_frame_state()` - Call every frame

### `mouse.rs` - Mouse Input

#### State Tracking
- Position (Vec2)
- Delta movement (Vec2)
- Wheel delta (f32)
- Button states (left, right, middle)
- Just pressed/released flags

#### Integration
Event listeners → Update state → Clear at frame end

---

## 📦 **6. Assets** (`src/assets/`)

### `mesh.rs` - Procedural Geometry

#### Cube
- 24 vertices (4 per face, unique normals)
- 36 indices (6 faces × 2 triangles × 3 vertices)

#### Sphere
- Parametric generation (stacks × slices)
- Smooth normals
- Configurable resolution

#### Quad
- 4 vertices, 2 triangles
- Used for billboarded particles

---

## 🌍 **7. Scene Management** (`src/scene/`)

### Scene Structure
```rust
Scene {
    entities: EntityManager
}
```

### Operations
- `update(engine)` - Updates physics and lifetimes
- `clear()` - Removes all entities

---

## 🔄 **Main Loop Flow**

```
1. Initialize Engine
   ├─ Setup WebGPU
   ├─ Create shaders
   ├─ Build pipeline
   └─ Initialize scene

2. Game Loop (every frame)
   ├─ Update Time
   │  ├─ Calculate delta_time
   │  └─ Update FPS counter
   │
   ├─ Update
   │  ├─ Process input
   │  ├─ Update camera
   │  ├─ Update scene (physics, lifetimes)
   │  └─ Spawn new particles
   │
   ├─ Render
   │  ├─ Get current texture
   │  ├─ Update uniforms
   │  ├─ Begin render pass
   │  ├─ For each entity:
   │  │  ├─ Update model matrix
   │  │  └─ Draw mesh
   │  └─ Submit commands
   │
   └─ Clear Frame State
      ├─ Clear input deltas
      └─ Clear just_pressed flags

3. Request Next Frame
   └─ requestAnimationFrame
```

---

## ⚡ **Performance Optimizations**

### Memory
- **Generational handles** prevent dangling references
- **Pool allocators** for O(1) allocation
- **Arena allocators** for temporary allocations

### Rendering
- **GPU-side transformations** (vertex shader)
- **Indexed rendering** (reuse vertices)
- **Uniform buffers** (minimize CPU→GPU transfers)

### ECS
- **Component arrays** (cache-friendly)
- **Batch updates** (update all physics together)
- **Lazy deletion** (mark dead, cleanup later)

### Future Optimizations
- **Instanced rendering** (draw many particles in one call)
- **Compute shaders** (GPU-based physics)
- **Frustum culling** (don't draw off-screen objects)
- **LOD (Level of Detail)** (simpler meshes when far)

---

## 📊 **Data Flow**

```
User Input
  ↓
Input System (keyboard, mouse)
  ↓
Game Update
  ↓
ECS (Transform, Velocity, Lifetime)
  ↓
Scene Update (Physics, Lifecycle)
  ↓
Render System
  ↓
Camera (View-Projection Matrix)
  ↓
For Each Entity:
  ↓
  Transform → Model Matrix
  ↓
  Uniform Buffer (MVP matrices)
  ↓
  WebGPU Pipeline
  ↓
  Vertex Shader (Position Transform)
  ↓
  Fragment Shader (Color + Lighting)
  ↓
  Framebuffer
  ↓
Display
```

---

## 🎯 **Design Patterns**

### 1. Entity-Component-System (ECS)
- **Entities**: Just IDs
- **Components**: Data only
- **Systems**: Logic only
- **Benefits**: Flexible, performant, maintainable

### 2. Data-Oriented Design
- Components in arrays (not objects with pointers)
- Iterate over all of one type at once
- Cache-friendly memory access

### 3. Generational Indices
- Handle = Index + Generation
- Detect stale references
- Prevent use-after-free

### 4. Command Pattern
- GPU command buffers
- Record commands, submit batch
- Reduces API overhead

---

## 🚀 **Scaling to Millions**

### Current Architecture
- ✅ 1M entity capacity
- ✅ Component pools pre-allocated
- ✅ Batch spawning (100 particles/frame)
- ✅ Automatic cleanup (lifetime system)

### Bottlenecks
- ❌ One draw call per particle
- ❌ CPU-side matrix updates
- ❌ No spatial partitioning

### Solutions (Future)
- ✅ Instanced rendering (1 draw call for all)
- ✅ Compute shader physics (GPU-side)
- ✅ Octree/BVH (frustum culling)
- ✅ LOD system (less detail = more particles)

---

This architecture is designed for **learning** and **scalability**. Every component is built to be understood, modified, and extended!

