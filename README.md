# Leva – Local AI Agent with Web UI

A lightweight, local AI agent built on top of
[OpenClaw](https://docs.openclaw.ai/) that runs
entirely on your laptop with **no database**.

## Features

- **Web UI** – beautiful dark-themed chat
  interface with streaming responses
- **Two-server architecture** – LLM Gateway
  (port 8000) + Automation Engine (port 8001)
- **Tool calling** – file read/write, 3-layer
  markdown memory, with multi-round tool loops
- **SSE streaming** – tokens appear progressively
  in the browser as the LLM generates them
- **Session management** – persistent chat
  sessions saved to JSON files
- **File browser** – browse, view, and upload
  files through the web UI
- **Jobs panel** – view running/completed
  background jobs
- **Model switching** – change LLM model from
  the Settings panel
- **Gmail integration** – read inbox, send email,
  search via official Gmail API (fast, OAuth2)
- **Web research** – search Google and fetch pages
  via Chrome DevTools Protocol (CDP)
- **LinkedIn** – read feed and like posts via CDP
- **Perplexity fallback** – web research without
  browser via Perplexity API
- **3-Layer Memory Architecture**
  - Layer 1 (Soul): core identity, always loaded
  - Layer 2 (Memory): daily logs + topic summaries
  - Layer 3 (Reference): full document library
- **CLI fallback** – `./start cli` for terminal

## Quick Start

```bash
# 1. Clone & enter the repo
git clone git@github.com:lselector/leva-agent.git
cd leva-agent

# 2. (Recommended) Set up a private repo for secrets — see "Private Repo" section below
export LEVA_AGENT_PRIV_DIR=~/path/to/your-private-repo

# 3. Run setup (creates runtime directories, builds Rust binaries)
./setup

# 4. Add your API key to the private .env (or export it in your shell)
echo "ANTHROPIC_API_KEY=sk-ant-..." >> $LEVA_AGENT_PRIV_DIR/.env

# 5. Run the web UI
./start
# Opens at http://localhost:8000

# 6. Or run the CLI
./start cli
```

## Private Repo (Recommended)

Leva separates **code** (this repo, safe to push publicly) from **private data**
(secrets, memory, credentials) stored in a separate private repo.

### Setup

```bash
# 1. Create a private repo (e.g. on GitHub as a private repository)
git clone git@github.com:yourname/leva-agent-priv.git ~/leva-agent-priv

# 2. Export the env var — add this to your shell profile (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish, etc.)
export LEVA_AGENT_PRIV_DIR=~/leva-agent-priv
# Fish shell:
# set -gx LEVA_AGENT_PRIV_DIR ~/leva-agent-priv

# 3. Run ./setup — it will create the required dirs inside the private repo
./setup

# 4. Put your secrets in the private .env
#    The agent loads $LEVA_AGENT_PRIV_DIR/.env before the project .env,
#    so private values always take precedence.
cat >> $LEVA_AGENT_PRIV_DIR/.env <<'EOF'
ANTHROPIC_API_KEY=sk-ant-...
PERPLEXITY_API_KEY=pplx-...
GMAIL_CLIENT_ID=...
GMAIL_CLIENT_SECRET=...
GMAIL_REFRESH_TOKEN=...
EOF
```

### What lives where

| Directory / File | Repo |
|---|---|
| Source code, prompts, soul | `leva-agent/` (this repo) |
| `credentials/` (Gmail OAuth tokens) | `leva-agent-priv/` |
| `memory/` (daily logs, sessions, topics) | `leva-agent-priv/` |
| `.env` (API keys, secrets) | `leva-agent-priv/` |
| `reference/` (document library) | `leva-agent/` |

If `LEVA_AGENT_PRIV_DIR` is **not** set, everything falls back to this repo
(original single-repo behaviour).

## New API Endpoints (Server B :8001)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/gmail/inbox` | Get recent inbox emails |
| GET  | `/api/gmail/email/{id}` | Get full email |
| POST | `/api/gmail/send` | Send an email |
| POST | `/api/gmail/search` | Search emails |
| GET  | `/api/gmail/labels` | List Gmail labels |
| GET  | `/api/web/tabs` | List Chrome tabs |
| POST | `/api/web/search` | Google search via CDP |
| POST | `/api/web/fetch` | Fetch page content |
| POST | `/api/web/screenshot` | Screenshot a URL |
| POST | `/api/linkedin/feed` | LinkedIn feed posts |
| POST | `/api/linkedin/like` | Like a feed post |

### Setup: Gmail OAuth2
```bash
# 1. Download credentials from Google Cloud Console
#    → save as $LEVA_AGENT_PRIV_DIR/credentials/gmail_credentials.json
#      (or leva-agent/credentials/ if not using a private repo)

# 2. Add Gmail OAuth values to $LEVA_AGENT_PRIV_DIR/.env
GMAIL_CLIENT_ID=...
GMAIL_CLIENT_SECRET=...
GMAIL_REFRESH_TOKEN=...
```

### Setup: Web / LinkedIn via CDP
```bash
# Start Chrome with remote debugging
./chrome_debug_start
# Chrome must stay running while using web/LinkedIn tools
```

### Setup: Perplexity fallback
```bash
# Add to $LEVA_AGENT_PRIV_DIR/.env (or your shell)
PERPLEXITY_API_KEY=pplx-...
```

## Architecture

```
┌──────────────┐     ┌───────────────┐
│  Browser UI  │────▶│  Server A     │
│  (frontend/) │◀────│  LLM Gateway  │
│              │ SSE │  :8000        │
└──────────────┘     └─────┬─────────┘
                           │ HTTP
                    ┌──────▼────────┐
                    │  Server B     │
                    │  Automation   │
                    │  :8001        │
                    └───────────────┘
```

- **Server A** (port 8000): Serves the frontend,
  handles `/api/chat` (SSE streaming), session
  management, model switching, and dispatches
  tool calls to Server B.
- **Server B** (port 8001): File operations,
  background jobs, and browser automation
  (Playwright-powered: navigate, extract,
  screenshot, Gmail, LinkedIn).

## Directory Layout

```
leva-agent/                   ← this repo (safe to make public)
├── start                     # Launch script
├── setup                     # One-time setup script
├── .env                      # Non-secret defaults only (no API keys)
├── frontend/                 # Web UI (HTML/JS)
├── soul/                     # Layer 1 – Core identity (always loaded)
│   ├── soul.md
│   ├── agents.md
│   └── user.md
├── prompts/                  # System prompts
│   ├── SYSTEM.md
│   └── TOOLS.md
├── reference/                # Layer 3 – Document library
├── common/                   # Shared Rust library
│   └── src/
│       ├── config.rs         # Paths & env vars (reads LEVA_AGENT_PRIV_DIR)
│       ├── memory_store.rs   # Session storage
│       └── tools/            # Tool implementations
├── server_llm/               # Server A – LLM Gateway (:8000)
├── server_auto/              # Server B – Automation (:8001)
└── cli/                      # CLI REPL

leva-agent-priv/              ← separate private repo (keep secret)
├── .env                      # All API keys and secrets
├── credentials/              # OAuth tokens (Gmail)
│   ├── gmail_credentials.json
│   └── gmail_token.json
└── memory/                   # Layer 2 – Working memory
    ├── YYYY-MM-DD.md         # daily logs
    ├── sessions/             # chat sessions (JSON)
    └── topics/               # topic summaries
```

## API Endpoints

### Server A (LLM Gateway) — port 8000
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/chat` | Chat (SSE stream) |
| GET | `/api/chat/history` | List sessions |
| GET | `/api/chat/{id}` | Get session |
| DELETE | `/api/chat/{id}` | Delete session |
| GET | `/api/models` | List models |
| PUT | `/api/models/current` | Switch model |

### Server B (Automation) — port 8001
| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/files/list` | List files |
| POST | `/files/read` | Read a file |
| POST | `/files/write` | Write a file |
| POST | `/jobs/start` | Start a job |
| GET | `/jobs/list` | List all jobs |
| GET | `/jobs/status/{id}` | Job status |
| POST | `/browser/navigate` | Go to URL |
| POST | `/browser/extract` | Extract text |
| POST | `/browser/screenshot` | Screenshot |
| POST | `/browser/click` | Click element |
| POST | `/browser/type` | Type text |
| GET | `/browser/content` | Page HTML |
| POST | `/browser/close` | Close browser |
| POST | `/browser/gmail/inbox` | Gmail inbox |
| POST | `/browser/gmail/compose` | Compose |
| POST | `/browser/linkedin/feed` | Feed |
| POST | `/browser/linkedin/like` | Like post |

## 3-Layer Memory Architecture

### Layer 1 – Soul (Core Identity)
Files in `soul/` are read **every turn** and
injected into the system prompt.

### Layer 2 – Memory (Working Memory)
Short summaries in `memory/`:
daily logs + topic summaries.

### Layer 3 – Reference (Document Library)
Full documents in `reference/`.

## License

MIT
