# Mini-Claw – Minimal Local OpenClaw-Style Agent

A lightweight, local AI agent inspired by
[OpenClaw](https://docs.openclaw.ai/) that runs
entirely on your laptop with **no database**.

## Features

- **Tool calling** – file read/write, 3-layer
  markdown memory
- **3-Layer Memory Architecture**
  - Layer 1 (Soul): core identity, always loaded
  - Layer 2 (Memory): daily logs + topic summaries
  - Layer 3 (Reference): full document library
- **Short-term chat buffer** – in-RAM sliding
  window of recent messages
- **CLI REPL** – simple interactive loop

## Quick Start

```bash
# 1. Clone & enter the repo
git clone git@github.com:lselector/jarvis_lev.git
cd jarvis_lev

# 2. Create a virtual environment & install deps
python -m venv .venv
source .venv/bin/activate
pip install -e .

# 3. Configure your API key
cp .env.example .env
# edit .env and set OPENAI_API_KEY

# 4. Run the agent
python -m src.main
```

## Directory Layout

```
├── pyproject.toml
├── .env.example
├── soul/                  # Layer 1 – Soul
│   ├── soul.md            # core identity
│   ├── agents.md          # agent configuration
│   └── user.md            # user profile
├── memory/                # Layer 2 – Memory
│   ├── YYYY-MM-DD.md      # daily logs
│   └── topics/            # topic summaries
│       └── <topic>.md
├── reference/             # Layer 3 – Reference
│   └── <document>.md      # full documents
├── prompts/
│   ├── SYSTEM.md          # system prompt
│   └── TOOLS.md           # tool descriptions
├── src/
│   ├── main.py            # CLI entry point
│   ├── config.py          # paths & env vars
│   ├── models.py          # LLM wrapper (OpenAI)
│   ├── agent_loop.py      # main agent loop
│   ├── channels/
│   │   └── cli.py         # REPL channel
│   ├── tools/
│   │   ├── base.py        # Tool protocol
│   │   ├── files.py       # file_read/write
│   │   ├── memory_tools.py# 3-layer memory
│   │   └── registry.py    # tool → function map
│   └── memory_store/
│       └── short_term.py  # in-RAM buffer
└── myprompts/             # dev prompts
```

## 3-Layer Memory Architecture

### Layer 1 – Soul (Core Identity)
Files in `soul/` are read **every turn** and
injected into the system prompt. The agent
always knows its purpose, personality, and
who the user is.

- `soul.md` – identity and principles
- `agents.md` – agent configuration
- `user.md` – user profile and preferences

### Layer 2 – Memory (Working Memory)
Short (under 4KB) summaries and breadcrumbs
stored in `memory/`:

- `YYYY-MM-DD.md` – daily append-only logs
- `topics/<name>.md` – topic-specific summaries

### Layer 3 – Reference (Document Library)
Full documents and long-form content stored
in `reference/`. No size limit. Memory files
can point here as breadcrumbs.

## License

MIT
