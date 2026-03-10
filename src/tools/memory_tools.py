"""Read/write/search across 3-layer memory."""

from datetime import date
from pathlib import Path
from ..config import (
    SOUL_DIR,
    MEMORY_DIR,
    MEMORY_TOPICS_DIR,
    REFERENCE_DIR,
)


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
    """Search lines in daily + topic memory files."""
    results = []
    if not MEMORY_DIR.exists():
        return "no memory yet"
    for md in MEMORY_DIR.rglob("*.md"):
        text = md.read_text(encoding="utf-8")
        for line in text.splitlines():
            if query.lower() in line.lower():
                rel = md.relative_to(MEMORY_DIR)
                results.append(
                    f"{rel}: {line}"
                )
    return "\n".join(results) or "no matches"


# --------------------------------------------------------------
def soul_read() -> str:
    """Read all Layer 1 soul files."""
    parts = []
    if not SOUL_DIR.exists():
        return "no soul files found"
    for md in sorted(SOUL_DIR.glob("*.md")):
        text = md.read_text(encoding="utf-8")
        parts.append(
            f"--- {md.name} ---\n{text}"
        )
    return "\n\n".join(parts) or "empty soul"


# --------------------------------------------------------------
def memory_topic_write(
    topic: str, content: str
) -> str:
    """Write/overwrite a topic memory file."""
    MEMORY_TOPICS_DIR.mkdir(
        parents=True, exist_ok=True
    )
    fname = _sanitize_filename(topic)
    path = MEMORY_TOPICS_DIR / f"{fname}.md"
    path.write_text(content, encoding="utf-8")
    return f"wrote topic: {fname}.md"


# --------------------------------------------------------------
def memory_topic_read(topic: str) -> str:
    """Read a specific topic memory file."""
    fname = _sanitize_filename(topic)
    path = MEMORY_TOPICS_DIR / f"{fname}.md"
    if not path.exists():
        return f"topic '{topic}' not found"
    return path.read_text(encoding="utf-8")


# --------------------------------------------------------------
def memory_topic_list() -> str:
    """List all topic memory files."""
    if not MEMORY_TOPICS_DIR.exists():
        return "no topics yet"
    files = sorted(MEMORY_TOPICS_DIR.glob("*.md"))
    names = [f.stem for f in files]
    return "\n".join(names) or "no topics yet"


# --------------------------------------------------------------
def reference_read(name: str) -> str:
    """Read a reference document by name."""
    fname = _sanitize_filename(name)
    path = REFERENCE_DIR / f"{fname}.md"
    if not path.exists():
        return f"reference '{name}' not found"
    return path.read_text(encoding="utf-8")


# --------------------------------------------------------------
def reference_write(
    name: str, content: str
) -> str:
    """Write/overwrite a reference document."""
    REFERENCE_DIR.mkdir(
        parents=True, exist_ok=True
    )
    fname = _sanitize_filename(name)
    path = REFERENCE_DIR / f"{fname}.md"
    path.write_text(content, encoding="utf-8")
    return f"wrote reference: {fname}.md"


# --------------------------------------------------------------
def reference_list() -> str:
    """List all reference documents."""
    if not REFERENCE_DIR.exists():
        return "no references yet"
    files = sorted(REFERENCE_DIR.glob("*.md"))
    names = [f.stem for f in files]
    return "\n".join(names) or "no references yet"


# --------------------------------------------------------------
def reference_search(query: str) -> str:
    """Search lines in reference documents."""
    results = []
    if not REFERENCE_DIR.exists():
        return "no references yet"
    for md in REFERENCE_DIR.rglob("*.md"):
        text = md.read_text(encoding="utf-8")
        for line in text.splitlines():
            if query.lower() in line.lower():
                rel = md.relative_to(
                    REFERENCE_DIR
                )
                results.append(
                    f"{rel}: {line}"
                )
    return "\n".join(results) or "no matches"


# --------------------------------------------------------------
def _sanitize_filename(name: str) -> str:
    """Sanitize a name for use as filename."""
    clean = name.strip().lower()
    clean = clean.replace(" ", "-")
    safe = ""
    for ch in clean:
        if ch.isalnum() or ch in "-_":
            safe += ch
    return safe or "untitled"
