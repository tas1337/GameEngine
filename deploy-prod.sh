#!/bin/bash
set -e

echo "=========================================="
echo "  🚀 Deploying to Fly.io                 "
echo "  💰 Backend + Frontend = ONE container  "
echo "  💵 FREE TIER - $0/month!               "
echo "=========================================="
echo ""

# Check if flyctl is installed
if ! command -v fly &> /dev/null; then
    echo "❌ Error: flyctl is not installed."
    echo "📥 Install: https://fly.io/docs/hands-on/install-flyctl/"
    exit 1
fi

# Clean old builds
echo "🧹 Cleaning old builds..."
rm -f Cargo.lock
rm -rf target/ pkg/

# Check if app exists
if ! fly status 2>/dev/null; then
    echo "📦 First time setup..."
    echo "💡 Creating app with FREE tier (shared-cpu-1x, 256MB RAM)"
    fly launch --config fly.toml --copy-config --yes
else
    echo "📦 Deploying update..."
    fly deploy
fi

echo ""
echo "================================"
echo "✅ Deployment complete!"
echo ""
echo "🌐 Your app: https://$(fly info -j | grep -o '"Hostname":"[^"]*' | cut -d'"' -f4)"
echo ""
echo "💰 Cost: $0/month (FREE tier!)"
echo "🎮 Frontend + Backend in ONE container"
echo "================================"

