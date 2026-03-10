"""File read/write tools for the workspace."""

from pathlib import Path
from ..config import BASE_DIR


# --------------------------------------------------------------
def _safe_path(path: str) -> Path:
    """Resolve path and ensure it is inside workspace."""
    p = (BASE_DIR / path).resolve()
    if not str(p).startswith(str(BASE_DIR)):
        raise ValueError("Path outside workspace")
    return p


# --------------------------------------------------------------
def file_read(path: str) -> str:
    """Read a small text file from the workspace."""
    p = _safe_path(path)
    return p.read_text(encoding="utf-8")


# --------------------------------------------------------------
def file_write(path: str, content: str) -> str:
    """Overwrite or create a text file."""
    p = _safe_path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")
    return (
        f"wrote {len(content)} chars to {path}"
    )
