# Claude Code Context Management Principles

This document provides guidelines for initializing and maintaining efficient conversations with Claude Code, especially when working on large codebases or complex tasks that may exhaust the context window.

---

## 1. Keep Context Lean

- **Start with a concise summary**: Limit your initial context to 100–300 words.
- **Include only essentials**:
  - Project/task objective
  - Current approach and key decisions
  - Files/functions impacted
  - Latest error message or failing test
  - Next step or specific request

---

## 2. Use Targeted Prompts

- **Request unified diffs or patches** for specific files/functions.
- **Ask for short code blocks** (one function at a time).
- **Link to files or gists** instead of pasting entire files.
- **Share minimal logs**: Only the last 20–40 lines of error output.

---

## 3. Minimize Artifacts in Chat

- **Store long logs externally** (e.g., gist, pastebin).
- **Reference repo paths and commit SHAs** instead of pasting code.
- **Summarize previous discussion** in bullet points; avoid re-pasting.

---

## 4. Front-Load Constraints

- **State requirements at the top**:
  - Language and version (e.g., Rust 1.70, crate versions)
  - Key APIs or architectural constraints
  - Testing expectations

---

## 5. Work Iteratively

- **Request one change per message**:
  - Example: "Propose unified diff to switch State to SemanticSearchEngine."
- **Apply, build/test, share only new error snippets.**
- **Repeat in small steps.**

---

## 6. Reset Conversation When Needed

- **If context is exhausted or compaction fails**:
  1. Copy a brief summary of the current state.
  2. Start a new chat and paste the summary.
  3. Continue with targeted requests.

---

## 7. Prevent Context Overload

- **Maintain a living 'Working Summary' doc.**
- **Ask for diffs, not full rewrites.**
- **Limit code to one file or function per turn.**
- **Avoid pasting dependency lockfiles or generated code.**

---

## 8. Use Longer-Context Models When Available

- **Switch to models with larger windows for big refactors.**
- **Keep context tight even with bigger models.**

---

## 9. 90% Context Window Threshold Protocol

When context usage reaches **90% of the available window**:

### Automatic Actions
- **Claude Code should auto-compact** the conversation to free up space
- If compaction is insufficient, prepare for conversation reset

### Living Document Protocol
Maintain a **SESSION_NOTES.md** file in the project root that captures:

**Required fields for each 90% checkpoint:**
```markdown
## Session: [YYYY-MM-DD HH:MM]

### Context Status
- Context usage: XX% (XXXk/XXXk tokens)
- Compaction attempted: Yes/No

### Key Accomplishments
- [Bullet list of completed tasks]
- [Files modified with line ranges]
- [Build/test status]

### Important Insights
- [Technical decisions made]
- [Blockers encountered and solutions]
- [Architecture changes]

### Current State
- [What's working]
- [What's in progress]
- [Current error/issue if any]

### Next Steps
1. [Immediate next task]
2. [Following tasks in priority order]
3. [Future considerations]

### Files to Reference
- `path/to/file.rs:100-200` - [Brief description]
- `path/to/other.ts:50` - [Brief description]

---
```

### Workflow at 90% Threshold

1. **Claude Code alerts**: "Context at 90% - initiating checkpoint"
2. **Save session notes**: Create/append to `SESSION_NOTES.md` with timestamp
3. **Auto-compact**: Attempt conversation compaction
4. **If still >90%**: Prepare clean summary for new conversation
5. **Commit notes**: `git add SESSION_NOTES.md && git commit -m "checkpoint: Session notes at 90% context"`

### Benefits
- **Continuity**: Never lose progress when switching conversations
- **Team collaboration**: Anyone can pick up where you left off
- **Historical record**: Track evolution of solutions over time
- **Quick restart**: Paste last session note to resume efficiently

---

## Quick Restart Checklist

1. Copy a 100–300 word summary of the current state.
2. Start a new chat and paste the summary.
3. Ask for unified diffs for one specific change.
4. Apply, build/test, paste only the short error snippet.
5. Repeat in small steps.

---

By following these principles, you'll maintain efficient, actionable conversations and avoid exhausting Claude Code's context window.

---
---

# Fast Browser Search - Project Context

## Overview

Fast Browser Search is a unified browser history search tool with semantic search capabilities, combining data from multiple browsers (Chrome, Safari, Arc, etc.) with NLP-enhanced search.

## Architecture

### Backend (Rust + Axum)
- **Main entry**: `src/main.rs` - SemanticApiServer with Axum routing
- **Search engines**:
  - `src/search/semantic.rs` - **SemanticSearchEngine** (current, NLP-enabled)
  - `src/search/simple.rs` - SimpleSearchEngine (legacy, keyword-based)
- **Database**: `src/db/simple_storage.rs` - SimpleStorage (in-memory + SQLite)
- **NLP modules**: `src/nlp/` - embeddings, keyword extraction, site mapping
- **Browser extractors**: `src/browser/` - Chrome, Safari, Arc history parsing

### Frontend (React + TypeScript)
- `frontend/src/` - React app with Tailwind CSS
- API client connects to `http://localhost:3000`

## Tech Stack

**Backend:**
- Rust 1.70+ with Tokio async runtime
- Axum web framework (v0.7)
- SimpleStorage (in-memory search with SQLite persistence)
- NLP: whatlang, rust-stemmers, custom TF-IDF embeddings

**Frontend:**
- React with TypeScript
- Vite dev server (port 5173)
- Tailwind CSS

## API Endpoints

### Standard Endpoints (use SemanticSearchEngine internally)
- `POST /api/search` - Search with filters
- `GET /api/suggest?query=...` - Search suggestions
- `GET /api/popular` - Most visited URLs
- `GET /api/domains` - All indexed domains
- `GET /api/related?url=...&limit=10` - Related URLs
- `POST /api/index` - Trigger re-indexing

### Semantic-Specific Endpoints
- `POST /api/semantic/search` - Semantic search (with `use_semantic` flag)
- `GET /api/semantic/similar?url=...&limit=10` - Find similar pages via embeddings
- `GET /api/semantic/topics` - Topic analysis
- `GET /api/semantic/sites` - Site summaries with categories

### Health
- `GET /health` - Health check

## Common Commands

### Build & Run
```bash
# Backend (Rust)
cargo build                    # Debug build
cargo build --release          # Production build
cargo run                      # Run with hot reload
RUST_LOG=debug cargo run      # Run with debug logging

# Frontend (React)
cd frontend
npm install                    # Install dependencies
npm run dev                    # Start dev server (localhost:5173)
npm run build                  # Production build
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Development Scripts
```bash
./run-backend.sh              # Start backend with env vars
```

## Key File Locations

- **Main server**: `src/main.rs:13-56` - Entry point, SemanticApiServer
- **Router**: `src/main.rs:95-118` - API route definitions
- **Handlers**: `src/main.rs:132-391` - Request handlers
- **SemanticSearchEngine**: `src/search/semantic.rs`
  - `new()` - Initialize with embedding model
  - `index_all_browsers()` - Index browser history
  - `semantic_search()` - Search with embeddings
  - `find_similar()` - Vector similarity search
  - `search()`, `get_domains()`, `get_popular_urls()`, `get_related_urls()` - Standard interface
- **Database trait**: `src/db/mod.rs:44-50` - HistoryDatabase trait
- **Browser extractors**: `src/browser/` - Chrome, Safari, Arc parsers

## Development Workflow

1. **Make changes** to Rust code in `src/`
2. **Build**: `cargo build` (check for errors)
3. **Test**: `cargo test` (run test suite)
4. **Run**: Backend via `./run-backend.sh`, Frontend via `npm run dev`
5. **API testing**: Use curl or frontend at `http://localhost:5173`

## Current State

- **Search engine**: Using **SemanticSearchEngine** (NLP-enabled)
- **State wiring**: `Arc<Arc<SemanticSearchEngine>>` passed to handlers
- **Endpoints**: All standard + semantic-specific routes active
- **Build status**: ✅ Compiles successfully (0 errors, warnings only)

## Common Patterns

### Adding a new endpoint
1. Define handler function in `src/main.rs`
2. Add route in `build_router()` method
3. Use `State<Arc<Arc<SemanticSearchEngine>>>` extractor
4. Return `impl IntoResponse`

### Modifying search behavior
- Edit `SemanticSearchEngine` methods in `src/search/semantic.rs`
- Methods use async/await with `Result<T>` returns
- Access DB via `self.db.method().await?`

### Working with embeddings
- Embedding generation: `src/nlp/embeddings.rs`
- Vector index: `VectorIndex::search()` for similarity
- Batch processing for performance (100 entries/batch)

## Environment Variables

```bash
API_PORT=3000                  # Backend API port
RUST_LOG=debug                 # Logging level (trace, debug, info, warn, error)
```

## Notes for AI Assistants

- **Primary codebase language**: Rust (backend) + TypeScript (frontend)
- **Async patterns**: All DB/search operations are async with Tokio
- **Error handling**: Use `anyhow::Result<T>` for errors
- **State management**: `Arc<SemanticSearchEngine>` shared across handlers
- **Testing**: Focus on handler logic and search accuracy
- **Compilation**: Always verify with `cargo build` before committing
