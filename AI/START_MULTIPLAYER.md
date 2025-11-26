# 🎮 Multiplayer Setup Guide

## Quick Start (2 Steps)

### 1️⃣ Start the Backend Server (Terminal 1)

```bash
cd backend
chmod +x run-dev.sh
./run-dev.sh
```

**You should see:**
```
🚀 Starting multiplayer server on ws://localhost:9001
   Compiling game_server v0.1.0
    Finished release [optimized] target(s) in X.XXs
     Running `target/release/game_server`
🎮 Game Server started on ws://0.0.0.0:9001
🌍 Ready for connections!
```

---

### 2️⃣ Start the Frontend (Terminal 2)

```bash
# Go back to root directory
cd ..

# Run the dev script
./run-dev.sh
```

**Open multiple browsers:**
- Browser 1: http://localhost:8080
- Browser 2: http://localhost:8080
- Browser 3: http://localhost:8080 (etc.)

**You should see in console:**
```
🌐 Multiplayer enabled!
✅ Connected to game server
```

---

## Architecture

```
┌──────────────┐     WebSocket      ┌──────────────┐
│   Browser 1  │ ←──────────────→  │              │
├──────────────┤  ws://localhost:   │   Backend    │
│   Browser 2  │ ←───── 9001 ─────→ │   Server     │
├──────────────┤                     │  (port 9001) │
│   Browser 3  │ ←──────────────→  │              │
└──────────────┘                     └──────────────┘
```

---

## Features

✅ **Real-time multiplayer** - See other players move in real-time
✅ **Synchronized particles** - All players see the same particles
✅ **Shared sun/time** - Day/night cycle synced across all clients
✅ **Efficient networking** - Binary protocol with LZ4 compression
✅ **Scalable** - Lock-free architecture for millions of players

---

## Troubleshooting

### "Running solo mode" message?
- Make sure the backend server is running first
- Check that port 9001 is not blocked

### Can't connect?
- Restart backend server
- Clear browser cache
- Check console for errors

---

## Solo Mode

If you don't want multiplayer, just run the frontend without the backend:
```bash
./run-dev.sh
```

You'll see:
```
⚠️ Multiplayer server not available (running solo mode)
```

Everything still works, you just won't see other players!

