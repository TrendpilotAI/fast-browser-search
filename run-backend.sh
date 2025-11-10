#!/bin/bash

# Set up Rust environment
export PATH="$HOME/.cargo/bin:$PATH"

# Set environment variables
export FALKOR_HOST=localhost
export FALKOR_PORT=6379
export REDIS_URL=redis://localhost:6380
export ZEP_URL=http://localhost:8000
export GRAPHITI_URL=http://localhost:8001
export API_PORT=3000
export RUST_LOG=browser_history_search=info

echo "🚀 Starting Fast Browser Search Backend..."
echo "   API Server: http://localhost:3000"
echo "   WebSocket: ws://localhost:3000/ws"
echo ""

# Run the backend
cargo run --release