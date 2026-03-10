from datetime import date
from pathlib import Path
from ..config import MEMORY_DIR


def _today_file() -> Path:
    today = date.today().isoformat()
    MEMORY_DIR.mkdir(parents=True, exist_ok=True)
    return MEMORY_DIR / f"{today}.md"


def memory_append(text: str) -> str:
    """Append a note to today's memory markdown file."""
    f = _today_file()
    prefix = "" if f.exists() else f"# {date.today().isoformat()}\n\n"
    with f.open("a", encoding="utf-8") as fh:
        fh.write(prefix + f"- {text}\n")
    return "ok"


def memory_search(query: str) -> str:
    """Return lines from memory files that match the query."""
    results = []
    if not MEMORY_DIR.exists():
        return "no memory yet"
    for md in MEMORY_DIR.glob("*.md"):
        for line in md.read_text(encoding="utf-8").splitlines():
            if query.lower() in line.lower():
                results.append(f"{md.name}: {line}")
    return "\n".join(results) or "no matches"
