#!/bin/bash

echo "🚀 Starting Fast Browser Search..."
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if FalkorDB is running
echo -e "${YELLOW}Checking FalkorDB...${NC}"
if ! nc -z localhost 6379 2>/dev/null; then
    echo -e "${RED}❌ FalkorDB is not running on port 6379${NC}"
    echo "Please start FalkorDB with: docker run -p 6379:6379 falkordb/falkordb:latest"
    exit 1
else
    echo -e "${GREEN}✓ FalkorDB is running${NC}"
fi

# Check if Redis is running
echo -e "${YELLOW}Checking Redis...${NC}"
if ! nc -z localhost 6380 2>/dev/null; then
    echo -e "${RED}❌ Redis is not running on port 6380${NC}"
    echo "Please start Redis with: docker run -p 6380:6379 redis:latest"
    exit 1
else
    echo -e "${GREEN}✓ Redis is running${NC}"
fi

echo ""
echo -e "${GREEN}Starting backend server...${NC}"

# Set environment variables
export FALKOR_HOST=localhost
export FALKOR_PORT=6379
export REDIS_URL=redis://localhost:6380
export ZEP_URL=http://localhost:8000
export GRAPHITI_URL=http://localhost:8001
export API_PORT=3000
export RUST_LOG=browser_history_search=info

# Build and run the Rust backend
cargo build --release
cargo run --release &
BACKEND_PID=$!

# Wait for backend to start
echo "Waiting for backend to start..."
sleep 5

# Check if backend is running
if ! nc -z localhost 3000 2>/dev/null; then
    echo -e "${RED}❌ Backend failed to start${NC}"
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

echo -e "${GREEN}✓ Backend is running on http://localhost:3000${NC}"
echo ""

# Start frontend
echo -e "${GREEN}Starting frontend...${NC}"
cd frontend
npm run dev &
FRONTEND_PID=$!

echo ""
echo -e "${GREEN}🎉 Fast Browser Search is ready!${NC}"
echo ""
echo "📝 Services:"
echo "   • Backend API: http://localhost:3000"
echo "   • Frontend UI: http://localhost:5173"
echo "   • WebSocket: ws://localhost:3000/ws"
echo ""
echo "Press Ctrl+C to stop all services"

# Handle shutdown
trap "echo 'Shutting down...'; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit" INT

# Wait for processes
wait