"""Read/write/search markdown memory files."""

from datetime import date
from pathlib import Path
from ..config import MEMORY_DIR


# --------------------------------------------------------------
def _today_file() -> Path:
    """Return path to today's memory file."""
    today = date.today().isoformat()
    MEMORY_DIR.mkdir(parents=True, exist_ok=True)
    return MEMORY_DIR / f"{today}.md"


# --------------------------------------------------------------
def memory_append(text: str) -> str:
    """Append a note to today's memory file."""
    f = _today_file()
    today_str = date.today().isoformat()
    prefix = (
        "" if f.exists()
        else f"# {today_str}\n\n"
    )
    with f.open("a", encoding="utf-8") as fh:
        fh.write(prefix + f"- {text}\n")
    return "ok"


# --------------------------------------------------------------
def memory_search(query: str) -> str:
    """Return matching lines from memory files."""
    results = []
    if not MEMORY_DIR.exists():
        return "no memory yet"
    for md in MEMORY_DIR.glob("*.md"):
        text = md.read_text(encoding="utf-8")
        for line in text.splitlines():
            if query.lower() in line.lower():
                results.append(
                    f"{md.name}: {line}"
                )
    return "\n".join(results) or "no matches"
