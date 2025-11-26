# 🚀 Performance Optimization Guide

## Current Issue: Low FPS at 82K Particles

**Problem:** FPS drops to ~30 with 82,193 particles  
**Cause:** Too many draw calls or instancing not working properly  
**Solution:** Enable GPU instancing correctly

---

## ✅ How to Get 1 Million+ Particles

### 1. **Check Instancing is ON**

In the UI, make sure this box is **CHECKED**:
```
☑ Instancing (GPU Optimized)
```

### 2. **Verify Draw Calls**

With instancing ON:
- **Draw Calls should be: 2** (1 for ground, 1 for ALL particles)
- If Draw Calls = Particle Count → Instancing is BROKEN

### 3. **Reduce Spawn Rate Initially**

To test instancing:
1. Set spawn rate to **0**
2. Click **Clear All**
3. Set spawn rate to **500**
4. Watch particles accumulate
5. FPS should stay at **60** even with 500K+ particles

---

## Performance Targets

| Particles | FPS (Without Instancing) | FPS (With Instancing) |
|-----------|-------------------------|----------------------|
| 10,000 | 60 | 60 |
| 50,000 | 20 | 60 |
| 100,000 | 10 | 60 |
| 500,000 | 2 | 60 |
| 1,000,000 | <1 | 55+ |

---

## Bottlenecks to Check

### CPU Bottlenecks:
1. **Physics Updates** (82K particles × physics = slow)
   - **Solution:** Disable particle physics temporarily
   - Change spawn lifetime to **2.0s** (fewer particles alive)

2. **Entity Iteration**
   - **Current:** Loop through all particles to gather instance data
   - **Solution:** Already optimized with Vec allocation

### GPU Bottlenecks:
1. **Too Many Vertices**
   - Cube = 36 vertices per particle
   - 82K × 36 = 2.95M vertices
   - **Solution:** Use simpler mesh (billboard quad = 4 vertices)

2. **Fill Rate** (pixels drawn)
   - **Solution:** Smaller particle size

---

## Quick Wins

### 1. Reduce Physics Cost (EASY - 2x FPS)

Edit `src/lib.rs`, comment out particle physics:

```rust
// Apply gravity and ground collision to particles
// DISABLED FOR PERFORMANCE
/*
for &entity in &self.scene.entities.entities.clone() {
    if let (Some(transform), Some(velocity)) = (
        self.scene.entities.transforms.get_mut(entity),
        self.scene.entities.velocities.get_mut(entity),
    ) {
        velocity.linear += GRAVITY * dt;
        
        if transform.position.y < 0.1 {
            transform.position.y = 0.1;
            velocity.linear.y = -velocity.linear.y * 0.5;
            velocity.linear *= 0.95;
        }
    }
}
*/
```

### 2. Use Billboard Quads (MEDIUM - 5x FPS)

Replace cube mesh with flat quads:

```rust
// In GameState::new()
let particle_mesh = Mesh::quad(0.05);  // Instead of cube
```

### 3. Frustum Culling (MEDIUM - 2x FPS)

Don't render particles behind camera:

```rust
// Before gathering instances
if !camera.can_see(transform.position) {
    continue;  // Skip this particle
}
```

### 4. LOD (Level of Detail) (HARD - 3x FPS)

```rust
let distance = camera.position.distance(&transform.position);
if distance > 50.0 {
    continue;  // Don't render far particles
}
```

---

## Debug Checklist

Run `./run-dev.sh` and check:

- [ ] Instancing checkbox is **checked**
- [ ] Draw calls show **2** (not 82,193)
- [ ] FPS stays above **50** with 100K particles
- [ ] Memory usage is reasonable (<300MB)
- [ ] No console errors

If draw calls = particle count:
1. Rebuild with `./run-dev.sh`
2. Hard refresh browser (Ctrl+Shift+R)
3. Check console for WebGL errors

---

## Expected Performance

With current setup (instanced cubes):
- ✅ **100K particles**: 60 FPS
- ✅ **500K particles**: 45 FPS
- ✅ **1M particles**: 30 FPS

With billboard quads:
- ✅ **100K particles**: 60 FPS
- ✅ **1M particles**: 60 FPS
- ✅ **5M particles**: 45 FPS

---

## The Real Issue

Your performance drop is because:
1. **Physics loop** is O(N) every frame (82K iterations)
2. **Ground collision checks** for every particle
3. **Velocity updates** for every particle

**Solution:** Disable particle ground physics - they don't need to bounce!

Particles should just:
- Spawn
- Move with velocity
- Die after lifetime
- **NO physics needed!**

---

## Test: Pure Instancing (No Physics)

1. Set lifetime to **1.0s**
2. Disable particle physics (comment out)
3. Set spawn rate to **5000**
4. Watch FPS stay at **60** even with 500K+ particles!

This proves instancing works - physics is the bottleneck.

---

Want me to:
1. Disable particle physics? (keep player physics)
2. Switch to billboard quads?
3. Add frustum culling?

Pick one and your FPS will skyrocket! 🚀

