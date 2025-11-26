# 🧪 Testing Guide

## Prerequisites

Before you can run this engine, you need:

### 1. **Rust & Cargo**
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### 2. **wasm-pack**
```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Verify installation
wasm-pack --version
```

### 3. **Docker**
- Download from: https://www.docker.com/products/docker-desktop
- Verify: `docker --version`

### 4. **WebGPU-Compatible Browser**
This engine uses WebGPU, which requires a modern browser:

#### Chrome/Edge (Recommended)
- Chrome 113+ or Edge 113+
- Enable WebGPU flag:
  1. Go to `chrome://flags`
  2. Search for "Unsafe WebGPU"
  3. Enable it
  4. Restart browser

#### Firefox
- Firefox 116+ (with WebGPU enabled in about:config)

#### Safari
- Safari Technology Preview (experimental)

---

## 🚀 Option 1: Docker (Easiest)

### Step 1: Build & Run
```bash
# Make scripts executable (first time only)
chmod +x run-dev.sh deploy-prod.sh

# Build and run with Docker
./run-dev.sh
```

This will:
1. Build the Rust code to WASM
2. Create a Docker image with Nginx
3. Start a container on port 8080
4. Show logs

### Step 2: Open Browser
```
http://localhost:8080
```

You should see:
- Loading screen
- Then particles spawning and animating
- FPS counter in console (F12)

### Step 3: Stop Container
```bash
# Press Ctrl+C in terminal
# Or manually:
docker stop game-engine-dev
```

---

## 🔧 Option 2: Manual Build (For Development)

### Step 1: Build WASM
```bash
wasm-pack build --target web --release
```

This creates a `pkg/` directory with:
- `game_engine_bg.wasm` - Your compiled Rust code
- `game_engine.js` - JavaScript bindings

### Step 2: Serve Locally
You need a local server (can't use `file://` due to CORS).

#### Using Python
```bash
python3 -m http.server 8000
```

#### Using Node.js
```bash
npx serve .
```

#### Using Rust
```bash
cargo install basic-http-server
basic-http-server .
```

### Step 3: Open Browser
```
http://localhost:8000
```

---

## 🐛 Troubleshooting

### "WebGPU not supported"

#### Check Browser Support
```javascript
// Open browser console (F12) and run:
if (navigator.gpu) {
    console.log("✅ WebGPU supported!");
} else {
    console.log("❌ WebGPU not supported");
}
```

#### Solutions
1. Use Chrome 113+ or Edge 113+
2. Enable WebGPU flags (see Prerequisites)
3. Update your browser
4. Check if GPU drivers are up to date

### "cargo: command not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "wasm-pack: command not found"

Install wasm-pack:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Docker Build Fails

#### On Windows
- Make sure Docker Desktop is running
- Check WSL2 is enabled
- Try running as Administrator

#### On macOS/Linux
```bash
# Give executable permissions
chmod +x run-dev.sh

# Run with sudo if needed
sudo ./run-dev.sh
```

### "Port 8080 already in use"

#### Find what's using the port
```bash
# Windows
netstat -ano | findstr :8080

# macOS/Linux
lsof -i :8080
```

#### Stop the container
```bash
docker rm -f game-engine-dev
```

#### Or change the port
Edit `run-dev.sh`:
```bash
docker run -d --name game-engine-dev -p 8081:80 game-engine
```

### Canvas is Black

1. **Check console (F12)** - Look for errors
2. **Verify WebGPU** - Run the support check above
3. **Check shader compilation** - Errors appear in console
4. **Verify GPU adapter** - Some virtual machines don't support WebGPU

### Poor Performance / Low FPS

#### Check Particle Count
```javascript
// In console:
// Look for "Particles: X" in logs
```

#### Reduce Spawn Rate
Edit `src/lib.rs`:
```rust
// Change from:
if self.engine.frame_count % 1 == 0 {
    self.spawn_particle_burst(100);  // 100 per frame
}

// To:
if self.engine.frame_count % 10 == 0 {
    self.spawn_particle_burst(50);   // 50 per 10 frames
}
```

#### Check GPU Usage
- Integrated GPU? Lower spawn rate
- Older GPU? Reduce particle count
- Many browser tabs? Close some

---

## 📊 Performance Testing

### Test 1: Baseline
1. Run the engine
2. Wait 30 seconds
3. Check console for FPS
4. Note particle count

**Expected:** 60 FPS with ~10,000 particles

### Test 2: Stress Test
Edit `src/lib.rs` to spawn more:
```rust
if self.engine.frame_count % 1 == 0 {
    self.spawn_particle_burst(1000);  // 1000 per frame!
}
```

Rebuild and test. How many particles before FPS drops?

**Goal:** Learn your hardware limits

### Test 3: Lifetime Test
Change particle lifetime:
```rust
let lifetime = Lifetime::new(10.0);  // Live 10 seconds
```

More particles will accumulate. Test max capacity.

---

## 🧪 Manual Tests

### Camera Controls
- ✅ Left-click drag - Camera orbits around center
- ✅ Scroll wheel - Zoom in/out
- ✅ Camera never clips through origin

### Particle System
- ✅ Particles spawn at origin
- ✅ Particles move outward
- ✅ Particles rise then fall (gravity-like)
- ✅ Particles have random colors
- ✅ Particles disappear after lifetime expires

### Performance
- ✅ FPS stays above 30 with 10K particles
- ✅ No memory leaks (check Task Manager over time)
- ✅ Smooth animation

### Responsive Design
- ✅ Canvas fills viewport
- ✅ Resize window - canvas adapts
- ✅ Mobile-friendly (if testing on mobile)

---

## 📝 Testing Checklist

Before deploying to production, verify:

- [ ] Runs in Docker locally
- [ ] WebGPU supported in target browser
- [ ] No console errors
- [ ] FPS stable (>30)
- [ ] Particles visible and animated
- [ ] Camera controls work
- [ ] Window resize works
- [ ] Info panel displays correctly
- [ ] Build completes without warnings

---

## 🚀 Deploy to Production

Once local testing passes:

```bash
./deploy-prod.sh
```

This will:
1. Check for `flyctl` CLI
2. Initialize Fly.io app (first time)
3. Build Docker image
4. Deploy to Fly.io
5. Give you a live URL

### First Deployment
Follow prompts:
- App name: `rust-game-engine-yourname`
- Region: Choose closest to you (e.g., `ord` for Chicago)
- Resources: **shared-cpu-1x** with **256MB** (cheapest)

### Verify Deployment
```bash
fly status
fly logs
```

Visit your app URL (provided after deploy).

---

## 🔍 Advanced Testing

### Profile with Chrome DevTools

1. Open DevTools (F12)
2. Go to "Performance" tab
3. Click "Record"
4. Let it run 10 seconds
5. Stop recording
6. Analyze:
   - Scripting time (JavaScript/WASM)
   - Rendering time (GPU)
   - Idle time

### Memory Profiling

1. DevTools → Memory tab
2. Take heap snapshot
3. Run for 1 minute
4. Take another snapshot
5. Compare - should be stable (no leaks)

### WebGPU Profiling

Chrome flags → Enable "WebGPU Developer Features"
- Shows GPU time per draw call
- Buffer upload stats
- Shader compilation time

---

## 📧 Getting Help

If stuck:

1. **Check console** - Most errors appear there
2. **Read the error** - WebGPU errors are detailed
3. **Verify prerequisites** - Rust, wasm-pack, Docker
4. **Try manual build** - Isolate Docker issues
5. **Update everything** - Browser, Rust, wasm-pack

---

Happy testing! 🎮

Remember: This is a learning project. Break things, fix them, learn!

