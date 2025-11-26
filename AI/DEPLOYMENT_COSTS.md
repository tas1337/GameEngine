# 💰 Fly.io Deployment Costs

## Option 1: Solo Mode (FREE) ✅

**What you get:**
- ✅ Full 3D engine with millions of particles
- ✅ Physics, LOD, instancing, multi-core
- ✅ Dynamic skybox and clouds
- ✅ Camera controls
- ❌ No multiplayer

**Setup:**
```bash
./deploy-prod.sh
```

**Cost: $0/month** (uses Fly.io free tier)

---

## Option 2: With Multiplayer ($5-10/month)

**What you get:**
- ✅ Everything from Solo Mode
- ✅ Real-time multiplayer
- ✅ See other players
- ✅ Synchronized world state

**Setup:**

1. **Deploy Backend:**
```bash
cd backend
fly launch --config fly.toml --copy-config
fly deploy
```

2. **Update Frontend:**
   - Edit `src/lib.rs` line ~921
   - Change `let server_url = None;` to:
   ```rust
   let server_url = Some("wss://YOUR-APP-NAME.fly.dev:9001");
   ```

3. **Deploy Frontend:**
```bash
./deploy-prod.sh
```

**Cost Breakdown:**
- Backend: ~$2-5/month (shared-cpu-1x, 256MB RAM)
- Frontend: $0 (free tier)
- **Total: $2-5/month**

---

## Option 3: High Traffic Multiplayer ($50-100/month)

For 100+ concurrent players:

**Backend (`backend/fly.toml`):**
```toml
[vm]
  size = "dedicated-cpu-1x"  # 1 dedicated CPU
  memory = "2gb"

[scaling]
  min_machines = 2
  max_machines = 5
```

**Cost:** ~$50-100/month depending on usage

---

## Recommended: Start with Solo Mode (FREE)

You can always add multiplayer later! Just:

1. Deploy solo mode now (free)
2. Test and optimize
3. Add multiplayer when you need it

**Current setup is FREE by default** ✅

---

## How to Toggle Multiplayer

**Disable (Solo Mode - FREE):**
```rust
// src/lib.rs line ~921
let server_url = None;
```

**Enable (Local Testing):**
```rust
let server_url = Some("ws://localhost:9001");
```

**Enable (Production):**
```rust
let server_url = Some("wss://your-backend.fly.dev:9001");
```

Then rebuild and deploy!

