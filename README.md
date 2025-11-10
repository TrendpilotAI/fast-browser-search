# Fast Browser Search

A lightning-fast, unified browser history search tool that combines browsing data from multiple browsers with graph-based relationships, Redis caching, and conversational memory.

## Features

- **Multi-Browser Support**: Chrome, Safari, Arc, Comet, Genspark, Thorium
- **Graph-Based Storage**: FalkorDB for relationship tracking between URLs
- **Lightning-Fast Cache**: Redis for instant search results
- **Conversational Memory**: Zep/Graphiti integration for context-aware search
- **Real-time Search**: WebSocket support for instant results
- **Modern UI**: React with Tailwind CSS for responsive design
- **Auto-indexing**: Watches browser history files for automatic updates

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Chrome    │     │   Safari    │     │    Arc      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                    │
       └───────────────────┼────────────────────┘
                           │
                    ┌──────▼──────┐
                    │ Rust Backend │
                    │   Extractor  │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   ┌────▼────┐      ┌──────▼──────┐   ┌──────▼──────┐
   │FalkorDB │      │    Redis    │   │Zep/Graphiti│
   │ (Graph) │      │   (Cache)   │   │  (Memory)  │
   └─────────┘      └─────────────┘   └─────────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           │
                    ┌──────▼──────┐
                    │  REST/WS API │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │React Frontend│
                    └─────────────┘
```

## Prerequisites

- Rust (1.70+)
- Node.js (18+)
- FalkorDB running locally (default port 6379)
- Redis running locally (default port 6380)
- Zep server (optional, default port 8000)
- Graphiti server (optional, default port 8001)

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/TrendpilotAI/fast-browser-search.git
cd fast-browser-search
```

### 2. Install and start database services

```bash
# Start FalkorDB (using Docker)
docker run -p 6379:6379 falkordb/falkordb:latest

# Start Redis
docker run -p 6380:6379 redis:latest

# (Optional) Start Zep
docker run -p 8000:8000 ghcr.io/getzep/zep:latest

# (Optional) Start Graphiti
# Follow Graphiti setup instructions
```

### 3. Build and run the Rust backend

```bash
# Build the backend
cargo build --release

# Run with environment variables
FALKOR_HOST=localhost \
FALKOR_PORT=6379 \
REDIS_URL=redis://localhost:6380 \
ZEP_URL=http://localhost:8000 \
GRAPHITI_URL=http://localhost:8001 \
API_PORT=3000 \
cargo run --release
```

### 4. Install and run the React frontend

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

The application will be available at http://localhost:5173

## Usage

### Basic Search

1. Open the web interface at http://localhost:5173
2. Type your search query in the search bar
3. Results appear instantly with:
   - URL and title
   - Visit time and count
   - Source browser
   - Related URLs

### Advanced Features

- **Browser Filtering**: Click browser icons to filter results
- **Domain Search**: Click domain tags to search within domains
- **Real-time Updates**: Search results update as you type via WebSocket
- **Auto-suggestions**: Based on your search history
- **Popular URLs**: See your most visited sites
- **Re-indexing**: Click the refresh button to re-index all browser histories

## Configuration

### Environment Variables

```bash
# Backend configuration
FALKOR_HOST=localhost          # FalkorDB host
FALKOR_PORT=6379               # FalkorDB port
REDIS_URL=redis://localhost:6380  # Redis connection URL
ZEP_URL=http://localhost:8000     # Zep server URL
GRAPHITI_URL=http://localhost:8001 # Graphiti server URL
API_KEY=your_api_key           # Optional API key for Zep/Graphiti
API_PORT=3000                  # API server port

# Logging
RUST_LOG=debug                 # Log level (trace, debug, info, warn, error)
```

### Supported Browsers

The tool automatically detects and indexes history from:

- Google Chrome (all profiles)
- Safari
- Arc Browser
- Comet Browser
- Genspark Browser
- Thorium Browser

Browser history locations on macOS:
- Chrome: `~/Library/Application Support/Google/Chrome/*/History`
- Safari: `~/Library/Safari/History.db`
- Arc: `~/Library/Application Support/Arc/User Data/*/History`
- Others: Similar Chromium-based paths

## API Endpoints

### REST API

- `POST /api/search` - Search history
- `GET /api/suggest?query=term` - Get search suggestions
- `GET /api/popular` - Get popular URLs
- `GET /api/domains` - Get all domains
- `GET /api/related?url=URL` - Get related URLs
- `POST /api/index` - Trigger re-indexing
- `GET /health` - Health check

### WebSocket

- Connect to `ws://localhost:3000/ws`
- Send search queries for real-time results

## Development

### Project Structure

```
fast-browser-search/
├── src/
│   ├── main.rs           # Application entry point
│   ├── api/              # REST/WebSocket API
│   ├── browser/          # Browser history extractors
│   ├── db/               # Database connections
│   ├── memory/           # Zep/Graphiti integration
│   └── search/           # Search engine logic
├── frontend/
│   ├── src/
│   │   ├── App.tsx       # Main React component
│   │   └── index.css     # Tailwind CSS
│   └── package.json
└── Cargo.toml
```

### Adding New Browser Support

1. Create a new extractor in `src/browser/`
2. Implement the `BrowserExtractor` trait
3. Add to the extractor list in `src/browser/mod.rs`

### Performance Tips

- FalkorDB creates graph relationships for faster traversal
- Redis caches search results for 1 hour
- File watcher auto-indexes changes every 10 seconds
- WebSocket provides real-time search without page refresh

## Troubleshooting

### Permission Issues with Safari

Safari's history database may require elevated permissions:
```bash
# Grant terminal full disk access in System Preferences
# Security & Privacy > Privacy > Full Disk Access
```

### FalkorDB Connection Failed

Ensure FalkorDB is running:
```bash
docker ps | grep falkordb
```

### Browser History Not Found

Check if browsers are installed and have history:
```bash
ls ~/Library/Application\ Support/
```

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a pull request

## License

MIT License - see LICENSE file for details

## Acknowledgments

- FalkorDB for graph database
- Redis for caching
- Zep/Graphiti for conversational memory
- All browser vendors for history storage formats