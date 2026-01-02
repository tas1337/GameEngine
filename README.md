# 🎮 3D Game Engine - Built 100% From Scratch

A high-performance 3D game engine written entirely in Rust with WebGL, designed to handle **millions of particles** with no lag. Everything is built from scratch - no game engine frameworks, just raw WebGL bindings and custom systems.

## 🌐 Multiplayer Support
- **Real-time multiplayer** with WebSocket server
- See other players in real-time
- Synchronized particles, sun, and time of day
- Efficient binary protocol with compression
- See [START_MULTIPLAYER.md](START_MULTIPLAYER.md) for setup

## 🚀 Features

### ✅ **Foundation (Step 1) - COMPLETE**
- ✨ **Custom Math Library**
  - `Vec2`, `Vec3`, `Vec4` - Full vector operations
  - `Mat4` - 4x4 matrices with transformations
  - `Quat` - Quaternions for rotations
  - All operators overloaded, SIMD-friendly layout
  
- 🧠 **Memory Management**
  - Custom Arena allocator
  - Object Pool allocator
  - Generational handle system
  - Zero-copy buffer management

- ⚙️ **Core Engine Loop**
  - High-precision timing
  - FPS counter
  - Delta time calculation
  - `requestAnimationFrame` integration

### ✅ **Rendering System (Step 2) - COMPLETE**
- 🎨 **Raw WebGPU Context**
  - Direct GPU bindings (no wgpu crate)
  - Surface configuration
  - Adapter and device management
  
- 🖼️ **Shader Pipeline**
  - WGSL shader compilation
  - Custom vertex/fragment shaders
  - Uniform buffer management
  - Bind groups and layouts

- 📦 **Buffer Management**
  - Vertex buffers
  - Index buffers
  - Uniform buffers
  - Dynamic buffer updates

- 📷 **Camera System**
  - Perspective projection
  - Look-at view matrix
  - Orbit controls
  - Zoom support

### ✅ **Entity Management (Step 3) - COMPLETE**
- 🎯 **Custom ECS (Entity Component System)**
  - Generational entity handles
  - Component pools
  - `Transform`, `Velocity`, `Color`, `Lifetime` components
  - Fast iteration over entities

### ✅ **Input System (Step 4) - COMPLETE**
- ⌨️ **Keyboard**
  - Key press/release detection
  - Just pressed/released events
  - Frame state management

- 🖱️ **Mouse**
  - Position tracking
  - Delta movement
  - Button states
  - Scroll wheel support

### ✅ **Asset Pipeline (Step 5) - COMPLETE**
- 🔷 **Procedural Mesh Generation**
  - Cube mesh
  - Sphere mesh (with subdivisions)
  - Quad mesh
  - Custom vertex format

### ✅ **Scene Management (Step 6) - COMPLETE**
- 🌍 **Scene Graph**
  - Entity management
  - Physics updates
  - Lifetime tracking
  - Batch rendering

## 📁 Project Structure

```
GameEngine/
├── src/
│   ├── math/              # Custom math library
│   │   ├── vec2.rs       # 2D vectors
│   │   ├── vec3.rs       # 3D vectors
│   │   ├── vec4.rs       # 4D vectors
│   │   ├── mat4.rs       # 4x4 matrices
│   │   ├── quat.rs       # Quaternions
│   │   └── mod.rs
│   │
│   ├── core/              # Core engine systems
│   │   ├── memory.rs     # Custom allocators
│   │   ├── engine.rs     # Main loop
│   │   └── mod.rs
│   │
│   ├── renderer/          # Rendering system
│   │   ├── gpu_context.rs # WebGPU setup
│   │   ├── buffer.rs      # GPU buffers
│   │   ├── shader.rs      # Shader management
│   │   ├── pipeline.rs    # Render pipeline
│   │   ├── camera.rs      # Camera system
│   │   └── mod.rs
│   │
│   ├── ecs/               # Entity Component System
│   │   ├── component.rs   # Component definitions
│   │   ├── entity.rs      # Entity manager
│   │   └── mod.rs
│   │
│   ├── input/             # Input handling
│   │   ├── keyboard.rs    # Keyboard input
│   │   ├── mouse.rs       # Mouse input
│   │   └── mod.rs
│   │
│   ├── assets/            # Asset management
│   │   ├── mesh.rs        # Mesh generation
│   │   └── mod.rs
│   │
│   ├── scene/             # Scene management
│   │   └── mod.rs
│   │
│   └── lib.rs             # Main entry point
│
├── Cargo.toml             # Rust dependencies (minimal!)
├── Dockerfile             # Multi-stage build
├── index.html             # Responsive HTML
├── run-dev.sh            # Local development script
├── deploy-prod.sh        # Fly.io deployment
└── README.md             # You are here!
```

## 🛠️ Technology Stack

- **Language**: Rust 🦀
- **Graphics API**: WebGPU (raw bindings)
- **Target**: WebAssembly (WASM)
- **Deployment**: Docker + Fly.io
- **Dependencies**: ONLY wasm-bindgen, js-sys, web-sys (no game engine crates!)

## 🚀 Quick Start

### Prerequisites
- Docker installed
- For deployment: Fly.io CLI (`flyctl`)

### Local Development

```bash
# Build and run in Docker
chmod +x run-dev.sh
./run-dev.sh
```

Visit `http://localhost:8080` to see your engine in action!

### Production Deployment

```bash
# Deploy to Fly.io (cheapest tier)
chmod +x deploy-prod.sh
./deploy-prod.sh
```

## 🎮 Controls

- **Left Mouse Button + Drag**: Rotate camera around scene
- **Scroll Wheel**: Zoom in/out
- **Open Browser Console**: See FPS and particle count

## 🔥 Performance

This engine is designed for extreme performance:

- ✅ Handles **millions of particles** simultaneously
- ✅ 60 FPS on modern hardware
- ✅ Zero-copy GPU buffer updates
- ✅ Custom memory allocators
- ✅ Efficient ECS architecture
- ✅ WASM optimized with LTO

## 📚 Learning Resources

This engine is built entirely from scratch to maximize learning:

- **Math**: All vector/matrix operations implemented manually
- **WebGPU**: Direct bindings, no abstraction layers
- **ECS**: Custom entity-component system
- **Memory**: Custom allocators (Arena, Pool, Generational)
- **Shaders**: WGSL written by hand

## 🎯 Roadmap

### Completed ✅
1. ✅ Foundation (Math, Memory, Core Loop)
2. ✅ Rendering System (WebGPU, Shaders, Buffers, Camera)
3. ✅ Entity Management (ECS)
4. ✅ Input System
5. ✅ Asset Pipeline
6. ✅ Scene Management
7. ✅ Networking (multiplayer)
8. ✅ Instanced rendering for better particle performance

### Future Enhancements
- [ ] Compute shaders for GPU-based particle physics
- [ ] Advanced lighting (PBR)
- [ ] Better Shadow mapping
- [ ] Post-processing effects
- [ ] Audio system

## 📝 License

This is a learning project - feel free to use it however you want!

## 🙏 Acknowledgments

Built with ❤️ and Rust 🦀

**No frameworks. No shortcuts. Just learning.**

---

Made by a developer who wanted to understand how game engines *really* work.

