# Jarvis Rust Conversion Plan

## Goal
Rewrite jarvis_lev from Python (FastAPI + uvicorn) to Rust (Axum + Tokio),
preserving all functionality: two-server architecture, SSE streaming, tool
calling, CDP browser automation, Gmail API, and the 3-layer memory system.

---

## Technology Choices

| Python (current)         | Rust replacement          |
|--------------------------|---------------------------|
| FastAPI + uvicorn        | axum + tokio              |
| httpx / requests         | reqwest                   |
| SSE (sse-starlette)      | axum::response::Sse       |
| openai Python SDK        | async-openai crate        |
| pychrome (CDP)           | Custom WebSocket (tokio-tungstenite) |
| google-api-python-client | reqwest + oauth2 crate    |
| python-dotenv            | dotenvy                   |
| serde/JSON               | serde + serde_json        |
| File I/O                 | tokio::fs                 |
| Logging                  | tracing + tracing-subscriber |

---

## Phase 1 — Project Scaffold

- [ ] 1.1  Init Rust workspace: `cargo new jarvis_rust --name jarvis`
- [ ] 1.2  Create workspace `Cargo.toml` with two crates:
          `server_llm` (port 8000) and `server_auto` (port 8001)
- [ ] 1.3  Add shared `common` crate for types, config, tools
- [ ] 1.4  Add dependencies: axum, tokio, serde, serde_json, dotenvy,
          reqwest, async-openai, tracing, tower-http, uuid
- [ ] 1.5  Copy `frontend/` and `soul/` as-is (static files, no changes needed)
- [ ] 1.6  Copy `prompts/`, `memory/`, `reference/` as-is (data dirs)
- [ ] 1.7  Create `rust_version/start` shell script (cargo run)
- [ ] 1.8  Create `rust_version/setup` shell script (cargo build)

---

## Phase 2 — Config & Environment

- [ ] 2.1  Port `src/config.py` → `common/src/config.rs`
          (load .env, define path constants, model defaults)
- [ ] 2.2  Define shared types in `common/src/types.rs`
          (ChatMessage, Session, Tool, ToolCall, StreamChunk)

---

## Phase 3 — Server LLM (port 8000)

- [ ] 3.1  Port `server_llm/app.py` → `server_llm/src/main.rs`
          (axum router, static file serving, CORS, startup)
- [ ] 3.2  Port `session_store.py` → `session_store.rs`
          (in-memory HashMap<String, Session> behind Arc<RwLock>)
- [ ] 3.3  Port `routes_models.py` → `routes_models.rs`
          (GET /api/models, PUT /api/models/current)
- [ ] 3.4  Port `streaming.py` → `streaming.rs`
          (call OpenAI streaming API, yield SSE chunks)
- [ ] 3.5  Port `routes_chat.py` → `routes_chat.rs`
          (POST /api/chat — SSE stream, GET/DELETE session routes)
- [ ] 3.6  Port `tool_dispatch.py` → `tool_dispatch.rs`
          (parse tool_calls from LLM, dispatch to server_auto via HTTP)

---

## Phase 4 — Tools

- [ ] 4.1  Port `tools/base.py` → `common/src/tools/base.rs`
          (Tool trait: name, description, parameters schema, execute)
- [ ] 4.2  Port `tools/registry.py` → `common/src/tools/registry.rs`
          (register tools, build JSON schema for LLM)
- [ ] 4.3  Port `tools/files.py` → `common/src/tools/files.rs`
          (read_file, write_file, list_files tools)
- [ ] 4.4  Port `tools/memory_tools.py` → `common/src/tools/memory.rs`
          (read_memory, write_memory, append_memory tools)
- [ ] 4.5  Port `memory_store/short_term.py` → `common/src/memory_store.rs`
          (in-memory per-session message history)

---

## Phase 5 — Server Auto (port 8001)

- [ ] 5.1  Port `server_auto/app.py` → `server_auto/src/main.rs`
          (axum router, startup)
- [ ] 5.2  Port `routes_files.py` → `routes_files.rs`
          (GET /files/list, POST /files/read, POST /files/write)
- [ ] 5.3  Port `routes_jobs.py` + `job_runner.py` → `routes_jobs.rs`
          (POST /jobs/start, GET /jobs/list, GET /jobs/status/{id},
           spawn tokio tasks, track via Arc<Mutex<HashMap>>)

---

## Phase 6 — CDP Browser

- [ ] 6.1  Port `cdp_browser/client.py` → `cdp_browser/src/client.rs`
          (WebSocket connection to Chrome via tokio-tungstenite,
           send/receive JSON-RPC CDP messages)
- [ ] 6.2  Port `cdp_browser/actions.py` → `cdp_browser/src/actions.rs`
          (navigate, extract_text, screenshot, click, type_text)
- [ ] 6.3  Port `cdp_browser/routes.py` → `cdp_browser/src/routes.rs`
          (axum routes: /browser/navigate, /browser/extract, etc.)
- [ ] 6.4  Port `cdp_browser/perplexity.py` → `cdp_browser/src/perplexity.rs`
          (Perplexity API fallback via reqwest)

---

## Phase 7 — Gmail API

- [ ] 7.1  Port `gmail_api/auth.py` → `gmail_api/src/auth.rs`
          (OAuth2 flow using oauth2 crate, save/load token.json)
- [ ] 7.2  Port `gmail_api/client.py` → `gmail_api/src/client.rs`
          (Gmail REST API calls via reqwest with Bearer token)
- [ ] 7.3  Port `gmail_api/routes.py` → `gmail_api/src/routes.rs`
          (POST /api/gmail/inbox, /send, /search, GET /labels, /email/{id})

---

## Phase 8 — CLI Channel

- [ ] 8.1  Port `channels/cli.py` + `agent_loop.py` → `cli/src/main.rs`
          (terminal REPL, call LLM, handle tool loops, print output)
- [ ] 8.2  Wire into `start` script: `./start cli` runs CLI binary

---

## Phase 9 — Testing & Polish

- [ ] 9.1  Add unit tests for tools (files, memory)
- [ ] 9.2  Add integration tests for chat SSE endpoint
- [ ] 9.3  Verify frontend works identically against Rust servers
- [ ] 9.4  Update README with Rust setup instructions
- [ ] 9.5  Performance comparison: Python vs Rust latency/memory

---

## File Mapping Summary

```
python_version/src/           →  rust_version/src/
  config.py                   →    common/src/config.rs
  models.py                   →    common/src/models.rs
  agent_loop.py               →    cli/src/main.rs
  channels/cli.py             →    cli/src/cli.rs
  tools/base.py               →    common/src/tools/base.rs
  tools/registry.py           →    common/src/tools/registry.rs
  tools/files.py              →    common/src/tools/files.rs
  tools/memory_tools.py       →    common/src/tools/memory.rs
  memory_store/short_term.py  →    common/src/memory_store.rs
  server_llm/app.py           →    server_llm/src/main.rs
  server_llm/routes_chat.py   →    server_llm/src/routes_chat.rs
  server_llm/routes_models.py →    server_llm/src/routes_models.rs
  server_llm/session_store.py →    server_llm/src/session_store.rs
  server_llm/streaming.py     →    server_llm/src/streaming.rs
  server_llm/tool_dispatch.py →    server_llm/src/tool_dispatch.rs
  server_auto/app.py          →    server_auto/src/main.rs
  server_auto/routes_files.py →    server_auto/src/routes_files.rs
  server_auto/routes_jobs.py  →    server_auto/src/routes_jobs.rs
  server_auto/job_runner.py   →    server_auto/src/job_runner.rs
  cdp_browser/client.py       →    cdp_browser/src/client.rs
  cdp_browser/actions.py      →    cdp_browser/src/actions.rs
  cdp_browser/routes.py       →    cdp_browser/src/routes.rs
  cdp_browser/perplexity.py   →    cdp_browser/src/perplexity.rs
  gmail_api/auth.py           →    gmail_api/src/auth.rs
  gmail_api/client.py         →    gmail_api/src/client.rs
  gmail_api/routes.py         →    gmail_api/src/routes.rs
```
