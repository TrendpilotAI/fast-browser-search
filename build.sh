#!/bin/bash

# Local build script for Fast Browser Search
echo "🔨 Building Fast Browser Search..."

# Set up Rust environment
export PATH="$HOME/.cargo/bin:$PATH"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check Rust is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo not found in PATH${NC}"
    echo "Installing Rust locally..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo -e "${GREEN}✓ Using Rust: $(rustc --version)${NC}"

# Build the backend
echo -e "${YELLOW}Building Rust backend...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✓ Backend built successfully${NC}"
else
    echo -e "${RED}✗ Backend build failed${NC}"
    exit 1
fi

# Check if frontend dependencies are installed
if [ ! -d "frontend/node_modules" ]; then
    echo -e "${YELLOW}Installing frontend dependencies...${NC}"
    cd frontend && npm install && cd ..
fi

echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo "To run the application:"
echo "  1. Start databases (optional):"
echo "     docker run -p 6379:6379 falkordb/falkordb:latest"
echo "     docker run -p 6380:6379 redis:latest"
echo ""
echo "  2. Run backend:"
echo "     ./run-backend.sh"
echo ""
echo "  3. Run frontend (in another terminal):"
echo "     cd frontend && npm run dev"