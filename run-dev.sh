#!/bin/bash

echo "=========================================="
echo "  🎮 Starting Game Engine (Dev Mode)     "
echo "  🐳 Backend + Frontend in ONE container "
echo "=========================================="
echo ""

# Stop existing container if running
echo "🛑 Stopping old container..."
docker rm -f game-engine-dev 2>/dev/null || true

# Clean old builds
echo "🧹 Cleaning old builds..."
rm -f Cargo.lock
rm -rf target/ pkg/

# Build the image (backend + frontend)
echo "🔨 Building Docker image (backend + frontend)..."
docker build -t game-engine .

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Run the container with BOTH services
    echo "🚀 Starting container..."
    docker run -d --name game-engine-dev -p 8080:80 -p 9001:9001 game-engine
    
    echo ""
    echo "================================"
    echo "✅ Both services running!"
    echo "🌐 Frontend: http://localhost:8080"
    echo "🔌 Backend:  ws://localhost:9001"
    echo "📊 Press Ctrl+C to stop watching logs"
    echo "================================"
    echo ""
    
    # Show logs
    docker logs -f game-engine-dev
else
    echo "❌ Build failed! Check errors above."
    exit 1
fi

