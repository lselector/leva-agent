# Mini-Claw – Minimal Local OpenClaw-Style Agent

A lightweight, local AI agent inspired by [OpenClaw](https://docs.openclaw.ai/) that runs entirely on your laptop with **no database required**.

## Features

- **Tool calling** – file read/write, markdown memory append & search
- **Markdown long-term memory** – append-only daily notes stored as `memory/YYYY-MM-DD.md`
- **Short-term chat buffer** – in-RAM sliding window of recent messages
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
├── prompts/
│   ├── SYSTEM.md          # core system prompt
│   └── TOOLS.md           # tool descriptions
├── memory/                # auto-created daily markdown notes
├── src/
│   ├── main.py            # CLI entry point
│   ├── config.py          # paths & env vars
│   ├── models.py          # LLM wrapper (OpenAI)
│   ├── agent_loop.py      # main agent loop
│   ├── channels/
│   │   └── cli.py         # REPL channel
│   ├── tools/
│   │   ├── base.py        # Tool protocol
│   │   ├── files.py       # file_read / file_write
│   │   ├── memory_tools.py# memory_append / memory_search
│   │   └── registry.py    # tool name → function map
│   └── memory_store/
│       └── short_term.py  # in-RAM message buffer
└── myprompts/             # development prompts (internal)
```

## License

MIT
