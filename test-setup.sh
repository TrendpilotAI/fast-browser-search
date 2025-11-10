#!/bin/bash

echo "🧪 Testing Fast Browser Search Setup..."
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Test 1: Check Rust installation
echo -e "${YELLOW}1. Checking Rust installation...${NC}"
CARGO_PATH="$HOME/.cargo/bin/cargo"
if [ -f "$CARGO_PATH" ] || command -v cargo &> /dev/null; then
    if [ -f "$CARGO_PATH" ]; then
        echo -e "${GREEN}✓ Rust is installed ($($CARGO_PATH --version))${NC}"
        CARGO="$CARGO_PATH"
    else
        echo -e "${GREEN}✓ Rust is installed ($(cargo --version))${NC}"
        CARGO="cargo"
    fi
else
    echo -e "${RED}✗ Rust is not installed${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Test 2: Check Node.js installation
echo -e "${YELLOW}2. Checking Node.js installation...${NC}"
if command -v node &> /dev/null; then
    echo -e "${GREEN}✓ Node.js is installed ($(node --version))${NC}"
else
    echo -e "${RED}✗ Node.js is not installed${NC}"
    echo "Install from: https://nodejs.org/"
    exit 1
fi

# Test 3: Check if project compiles
echo -e "${YELLOW}3. Testing Rust compilation...${NC}"
if $CARGO build --release 2>/dev/null; then
    echo -e "${GREEN}✓ Backend compiles successfully${NC}"
else
    echo -e "${RED}✗ Backend compilation failed${NC}"
    echo "Run '$CARGO build' to see errors"
    exit 1
fi

# Test 4: Check frontend dependencies
echo -e "${YELLOW}4. Checking frontend dependencies...${NC}"
if [ -d "frontend/node_modules" ]; then
    echo -e "${GREEN}✓ Frontend dependencies installed${NC}"
else
    echo -e "${RED}✗ Frontend dependencies not installed${NC}"
    echo "Run: cd frontend && npm install"
    exit 1
fi

# Test 5: Check database connectivity
echo -e "${YELLOW}5. Checking database services...${NC}"
FALKOR_OK=false
REDIS_OK=false

if nc -z localhost 6379 2>/dev/null; then
    echo -e "${GREEN}✓ Port 6379 (FalkorDB) is accessible${NC}"
    FALKOR_OK=true
else
    echo -e "${YELLOW}⚠ FalkorDB not running on port 6379${NC}"
    echo "  Start with: docker run -p 6379:6379 falkordb/falkordb:latest"
fi

if nc -z localhost 6380 2>/dev/null; then
    echo -e "${GREEN}✓ Port 6380 (Redis) is accessible${NC}"
    REDIS_OK=true
else
    echo -e "${YELLOW}⚠ Redis not running on port 6380${NC}"
    echo "  Start with: docker run -p 6380:6379 redis:latest"
fi

echo ""
echo "📊 Test Summary:"
echo "  • Rust: ✅"
echo "  • Node.js: ✅"
echo "  • Backend Build: ✅"
echo "  • Frontend Deps: ✅"

if $FALKOR_OK && $REDIS_OK; then
    echo "  • Databases: ✅"
    echo ""
    echo -e "${GREEN}🎉 All tests passed! System is ready.${NC}"
    echo ""
    echo "To start the application, run: ./start.sh"
else
    echo "  • Databases: ⚠️ (optional services not running)"
    echo ""
    echo -e "${YELLOW}⚠ System partially ready. Start databases for full functionality.${NC}"
    echo ""
    echo "Quick start without databases (limited functionality):"
    echo "  1. Backend: cargo run --release"
    echo "  2. Frontend: cd frontend && npm run dev"
fi