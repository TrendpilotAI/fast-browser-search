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

## Claude MCP Agent Best Practices

When working with Claude MCP (Model Context Protocol) agents and tools:

- **Load tool definitions on demand**: Don't load all tool schemas up front—fetch only what's needed for the current task to minimize token usage.
- **Write agent workflows as code**: Use TypeScript or Python for maintainable, version-controlled agent logic instead of natural language prompts.
- **Filter and process raw results**: Clean and summarize raw tool outputs before returning to the model context—don't dump large JSON responses directly.
- **Persist intermediate state**: For long-running tasks, save checkpoints to disk or databases to enable resumability if conversations reset.
- **Tokenize all PII and sensitive info**: Never log raw passwords, API keys, or personal data—use tokenization or redaction.
- **Save reusable skills/scripts**: Document agent capabilities in SKILL.md files with clear usage examples and parameter descriptions.
- **Ensure sandboxed execution**: Run agent-generated code in isolated, monitored environments with appropriate security controls for reliability and safety.

---

## MCP Server Organization

### Filesystem Hierarchy Pattern

Structure MCP servers for progressive disclosure:

```
/servers/
  /gmail/
    send_email.ts
    search_inbox.ts
  /sheets/
    read_range.ts
    write_data.ts
  /salesforce/
    create_record.ts
```

**Benefits:**
- Tools discovered on-demand, not loaded upfront
- Reduces initial token overhead
- Enables tool search/discovery utilities
- Clearer separation of concerns

### Tool Design Principles

- **Lean definitions**: Only essential parameters and return types
- **Type safety**: Interface definitions for inputs/outputs
- **Single purpose**: Composable functions that do one thing well
- **Searchable**: Implement `search_tools()` utility for discovery

---

## Code Execution vs Direct Tool Calls: Decision Matrix

### Use Code Execution When:

- **High-volume data processing**: Filtering 10,000+ rows locally vs passing through context
- **Complex logic**: Loops, conditionals, error handling without model round-trips
- **Token efficiency critical**: Can reduce token usage by 90%+ in data pipelines
- **Privacy-sensitive operations**: Tokenize PII before reaching model context
- **Multi-step workflows**: Checkpoint intermediate state to filesystem

**Example**: Filter spreadsheet rows in execution environment instead of passing all through context

### Use Direct Tool Calls When:

- **Simple operations**: Single API calls or queries
- **Low data volume**: Results fit comfortably in context
- **Debugging required**: Easier to trace individual tool calls
- **Security uncertain**: Execution environment not fully sandboxed

### Tradeoff Analysis

**Code Execution Gains:**
- 98.7% token reduction (real case: 150k → 2k tokens)
- Lower latency (no model round-trips for loops)
- Better tool composition

**Operational Cost:**
- Requires secure sandboxing infrastructure
- Resource limits and monitoring needed
- More complex error handling

---

## Privacy-Preserving MCP Patterns

### Automatic PII Tokenization

Execute data transformations **before** model sees content:

```typescript
// In execution environment (not model context)
async function syncContactsWithTokenization(sheetData, salesforceAPI) {
  const tokenizedData = sheetData.map(row => ({
    email: tokenize(row.email),      // Real email never reaches model
    phone: tokenize(row.phone),      // Real phone never reaches model
    name: tokenize(row.name)         // Real name never reaches model
  }));

  // Model only sees tokens, not real PII
  return await salesforceAPI.createRecords(tokenizedData);
}
```

**Flow:**
1. Google Sheets → Execution Environment (real data)
2. Execution Environment → Tokenization (PII protected)
3. Tokenized data → Model context (safe)
4. Model logic → Execution Environment
5. Execution Environment → Salesforce (real data restored)

**Real PII flows from source to destination, never through model.**

---

## Code Execution: Resource Limits & Monitoring

### Required Safeguards

- **Execution timeout**: Max 5-10 minutes per run
- **Memory limits**: 512MB - 2GB depending on use case
- **CPU throttling**: Prevent runaway processes
- **Filesystem quotas**: Limit disk usage for checkpoints
- **Network isolation**: Whitelist only required external services

### Monitoring Checklist

- [ ] Log all code executions with timestamps
- [ ] Track token usage before/after execution
- [ ] Monitor resource consumption (CPU, memory, disk)
- [ ] Alert on execution failures or timeouts
- [ ] Audit file access patterns
- [ ] Track PII tokenization success rate

### Sandboxing Requirements

- Container-based isolation (Docker, gVisor, Firecracker)
- No access to host filesystem
- Restricted network egress
- Non-root user execution
- Read-only system directories

---

## MCP Efficiency Optimization

### Progressive Data Processing

**Anti-pattern:**
```javascript
// Loading all 10,000 rows into context
const allRows = await sheets.readRange('A1:Z10000');
const filtered = allRows.filter(row => row.status === 'active');
```

**Better:**
```javascript
// Filter in execution environment, return only results
const activeRows = await executeCode(`
  const data = await sheets.readRange('A1:Z10000');
  return data.filter(row => row.status === 'active');
`);
// Only ~100 rows reach model context instead of 10,000
```

### Checkpoint Long Workflows

```typescript
// Save intermediate results to filesystem
async function longRunningPipeline(data) {
  const step1 = await processPhase1(data);
  await fs.writeFile('/tmp/checkpoint1.json', JSON.stringify(step1));

  const step2 = await processPhase2(step1);
  await fs.writeFile('/tmp/checkpoint2.json', JSON.stringify(step2));

  return step2;
}
```

### Skill Persistence

Save frequently used functions for reuse:
```typescript
// Save as skill for future conversations
await saveSkill('filterActiveContacts', `
  async function filterActiveContacts(sheetRange) {
    const data = await sheets.readRange(sheetRange);
    return data.filter(row => row.status === 'active' && row.verified);
  }
`);
```

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
