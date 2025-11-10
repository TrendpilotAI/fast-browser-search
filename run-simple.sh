#!/bin/bash

# Set environment variables
export FALKOR_HOST=localhost
export FALKOR_PORT=6379
export REDIS_URL=redis://localhost:6380
export ZEP_URL=http://localhost:8000
export GRAPHITI_URL=http://localhost:8001
export API_PORT=3000
export RUST_LOG=browser_history_search=info

echo "🚀 Starting Fast Browser Search Backend (Simple Mode)..."
echo "   API Server: http://localhost:3000"
echo ""

# Build with simplified main
cp src/main_simple.rs src/main.rs.bak 2>/dev/null || true
cp src/main.rs src/main.rs.original 2>/dev/null || true

# Remove Redis cache calls from search module
sed -i.bak 's/cache\.invalidate_search_cache.*//g' src/search/mod.rs
sed -i.bak 's/.*cache\..*//g' src/search/mod.rs
sed -i.bak '/redis_cache/d' src/search/mod.rs

# Build
cargo build --release

# Run
cargo run --release