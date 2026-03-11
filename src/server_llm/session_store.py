"""Per-session message history with persistence."""

import json
from pathlib import Path
from typing import List, Dict, Optional
from ..config import MEMORY_DIR

SESSIONS_DIR = MEMORY_DIR / "sessions"


# --------------------------------------------------------------
class SessionStore:
    """Manage per-session chat histories."""

    def __init__(self):
        self._cache: Dict[str, List[dict]] = {}
        SESSIONS_DIR.mkdir(
            parents=True, exist_ok=True
        )

    # ----------------------------------------------------------
    def _path(self, sid: str) -> Path:
        """Return file path for a session."""
        safe = sid.replace("/", "_")
        return SESSIONS_DIR / f"{safe}.json"

    # ----------------------------------------------------------
    def _load(self, sid: str) -> List[dict]:
        """Load session from disk if not cached."""
        if sid in self._cache:
            return self._cache[sid]
        p = self._path(sid)
        if p.exists():
            data = json.loads(
                p.read_text(encoding="utf-8")
            )
            self._cache[sid] = data
            return data
        self._cache[sid] = []
        return self._cache[sid]

    # ----------------------------------------------------------
    def _save(self, sid: str):
        """Persist session to disk."""
        p = self._path(sid)
        p.write_text(
            json.dumps(
                self._cache[sid], indent=2
            ),
            encoding="utf-8",
        )

    # ----------------------------------------------------------
    def get_messages(
        self, sid: str
    ) -> List[dict]:
        """Get all messages for a session."""
        return list(self._load(sid))

    # ----------------------------------------------------------
    def add_message(
        self, sid: str, role: str, content: str
    ):
        """Add a message and persist."""
        msgs = self._load(sid)
        msgs.append(
            {"role": role, "content": content}
        )
        # Keep last 50 messages
        if len(msgs) > 50:
            self._cache[sid] = msgs[-50:]
        self._save(sid)

    # ----------------------------------------------------------
    def list_sessions(self) -> List[str]:
        """List all session IDs."""
        if not SESSIONS_DIR.exists():
            return []
        files = sorted(
            SESSIONS_DIR.glob("*.json")
        )
        return [f.stem for f in files]

    # ----------------------------------------------------------
    def get_session(
        self, sid: str
    ) -> Optional[List[dict]]:
        """Get session messages or None."""
        p = self._path(sid)
        if not p.exists() and sid not in self._cache:
            return None
        return self.get_messages(sid)

    # ----------------------------------------------------------
    def delete_session(self, sid: str) -> bool:
        """Delete a session."""
        self._cache.pop(sid, None)
        p = self._path(sid)
        if p.exists():
            p.unlink()
            return True
        return False


# Singleton instance
store = SessionStore()
