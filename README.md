# Leva – Local AI Agent with Web UI

A lightweight, local AI agent inspired by
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

# 2. Install dependencies
./setup

# 3. Configure your API key (if env var OPENAI_API_KEY is not set)
cp .env.example .env
# edit .env and set OPENAI_API_KEY

# 4. Run the web UI
./start
# Opens at http://localhost:8000

# 5. Or run the CLI
./start cli
```

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
#    → save as credentials/gmail_credentials.json
# 2. Run one-time auth flow
python -m src.gmail_api.auth
# Browser opens → log in → grant access → done
```

### Setup: Web / LinkedIn via CDP
```bash
# Start Chrome with remote debugging
./chrome_debug
# Chrome must stay running while using web/LinkedIn tools
```

### Setup: Perplexity fallback
```bash
# Add to .env
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
├── start                  # Launch script
├── pyproject.toml
├── frontend/              # Web UI
│   ├── index.html
│   ├── styles.css         # All styles here
│   ├── app.js             # Tab/session logic
│   ├── chat.js            # Chat + streaming
│   ├── files.js           # File browser
│   ├── jobs.js            # Jobs panel
│   ├── settings.js        # Model switching
│   ├── markdown.js        # MD → HTML
│   └── utils.js           # Helpers
├── soul/                  # Layer 1 – Soul
│   ├── soul.md
│   ├── agents.md
│   └── user.md
├── memory/                # Layer 2 – Memory
│   ├── YYYY-MM-DD.md      # daily logs
│   ├── sessions/          # chat sessions
│   └── topics/
├── reference/             # Layer 3 – Reference
├── prompts/
│   ├── SYSTEM.md
│   └── TOOLS.md
├── src/
│   ├── config.py          # paths & env vars
│   ├── models.py          # LLM wrapper
│   ├── agent_loop.py      # CLI agent loop
│   ├── server_llm/        # Server A
│   │   ├── app.py
│   │   ├── routes_chat.py
│   │   ├── routes_models.py
│   │   ├── session_store.py
│   │   ├── streaming.py
│   │   └── tool_dispatch.py
│   ├── server_auto/       # Server B
│   │   ├── app.py
│   │   ├── routes_files.py
│   │   ├── routes_jobs.py
│   │   ├── routes_browser.py
│   │   ├── job_runner.py
│   │   ├── browser_manager.py
│   │   ├── gmail_actions.py
│   │   └── linkedin_actions.py
│   ├── channels/
│   │   └── cli.py
│   ├── tools/
│   │   ├── base.py
│   │   ├── files.py
│   │   ├── memory_tools.py
│   │   └── registry.py
│   └── memory_store/
│       └── short_term.py
└── myprompts/             # dev prompts
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
