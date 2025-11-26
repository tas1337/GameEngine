# 🎮 Quick Start: Multiplayer

## Current Status

✅ **Frontend**: Has WebSocket client code (connects to backend)  
✅ **Backend**: Full multiplayer server created  
⚠️ **Connection**: Backend not running yet (single-player mode)

---

## How It Works Now

### Single-Player Mode (Current):
- Each browser window is independent
- Particles are client-side only
- No synchronization between windows
- Stats show: "👥 Players: 1 (solo)"

### Multiplayer Mode (When Backend Running):
- All browsers connect to same server
- Server controls particles (synchronized)
- Players see each other
- Stats show: "👥 Players: X"

---

## Enable Multiplayer

### Step 1: Start Backend Server

```bash
# Terminal 1 - Start multiplayer server
cd backend
cargo run --release
```

Server starts on: `ws://localhost:9001`

### Step 2: Rebuild Frontend

```bash
# Terminal 2 - Rebuild with multiplayer
./run-dev.sh
```

### Step 3: Open Multiple Browsers

```
http://localhost:8080  (Browser 1)
http://localhost:8080  (Browser 2)
http://localhost:8080  (Browser 3)
```

**You'll see:**
- 👥 Players: 3
- Other players' positions
- Synchronized particles
- Same sun/time for everyone

---

## What Gets Synchronized

✅ **Player Position** - See other players move  
✅ **Particles** - Same particles, same positions  
✅ **Sun/Time** - Day/night cycle synced  
✅ **Spawn Events** - Particles spawned by anyone appear for everyone  

❌ **NOT Synced** (yet):
- Camera rotation (only position)
- Player models (uses simple representation)

---

## Testing Multiplayer Locally

### Option 1: Multiple Browser Windows
```
Chrome Window 1: localhost:8080
Chrome Window 2: localhost:8080
Firefox Window:  localhost:8080
```

### Option 2: Multiple Devices
```
Computer 1: http://192.168.1.100:8080
Computer 2: http://192.168.1.100:8080
Phone:      http://192.168.1.100:8080
```

(Replace `192.168.1.100` with your local IP)

---

## Deploy to Production

### Backend (Fly.io):
```bash
cd backend
fly launch --name game-server
fly deploy
```

Gets: `wss://game-server.fly.dev`

### Frontend (Update URL):
```rust
// src/lib.rs
let server_url = "wss://game-server.fly.dev";  // Change this
```

### Frontend (Fly.io):
```bash
fly launch --name game-client
fly deploy
```

**Done!** Everyone can play at: `https://game-client.fly.dev`

---

## Troubleshooting

### "Players: 1 (solo)" always shows:
- Backend server not running
- Wrong URL in `src/lib.rs`
- Firewall blocking port 9001

### Can't see other players:
- Backend needs to broadcast player positions
- Check browser console for WebSocket errors

### Particles not synchronized:
- Spawn logic needs to go through server
- Currently client-side only

---

## Next Steps to Complete Multiplayer

1. **Render Other Players**:
```rust
// In render() function
for player in &self.network.remote_players {
    // Draw player capsule at player.position
}
```

2. **Update Stats**:
```javascript
document.getElementById('player-count').textContent = 
    `👥 Players: ${state.network.player_count}`;
```

3. **Sync Particles**:
   - Currently particles spawn client-side
   - Need to send spawn events through server
   - Server applies physics, sends to all

---

## Current Implementation

### Client (`src/network/mod.rs`):
✅ WebSocket connection  
✅ Send player updates (20 Hz)  
✅ Receive world updates  
✅ Sync time of day  

### Server (`backend/src/main.rs`):
✅ Accept connections  
✅ Broadcast to all players  
✅ Shared particle physics  
✅ 60 Hz tick rate  

### Missing:
- Render remote players in client
- Send particle spawns through server
- Display player count in UI

---

**Bottom Line**: Backend exists, client has code, just need to connect them!

Run `cargo run --release` in `backend/` directory to enable multiplayer! 🚀

