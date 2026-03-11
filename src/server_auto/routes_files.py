"""File operation endpoints for Server B."""

import os
from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional, List
from ..config import BASE_DIR

router = APIRouter(prefix="/files")

# Allowed root for file operations
WORKSPACE = BASE_DIR


# --------------------------------------------------------------
class FileReadRequest(BaseModel):
    """Request to read a file."""
    path: str


# --------------------------------------------------------------
class FileWriteRequest(BaseModel):
    """Request to write a file."""
    path: str
    content: str


# --------------------------------------------------------------
class FileListRequest(BaseModel):
    """Request to list files."""
    path: Optional[str] = "."
    recursive: Optional[bool] = False


# --------------------------------------------------------------
def _safe_path(rel_path: str):
    """Resolve path safely within workspace."""
    resolved = (WORKSPACE / rel_path).resolve()
    if not str(resolved).startswith(
        str(WORKSPACE.resolve())
    ):
        return None
    return resolved


# --------------------------------------------------------------
@router.post("/read")
async def file_read(req: FileReadRequest):
    """Read a text file."""
    path = _safe_path(req.path)
    if path is None:
        return {
            "error": "path outside workspace"
        }
    if not path.exists():
        return {"error": f"not found: {req.path}"}
    if not path.is_file():
        return {"error": "not a file"}
    try:
        content = path.read_text(
            encoding="utf-8"
        )
        return {
            "path": req.path,
            "content": content,
            "chars": len(content),
        }
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/write")
async def file_write(req: FileWriteRequest):
    """Write or create a text file."""
    path = _safe_path(req.path)
    if path is None:
        return {
            "error": "path outside workspace"
        }
    try:
        path.parent.mkdir(
            parents=True, exist_ok=True
        )
        path.write_text(
            req.content, encoding="utf-8"
        )
        return {
            "status": "ok",
            "path": req.path,
            "chars": len(req.content),
        }
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.get("/list")
async def file_list(
    path: str = ".",
    recursive: bool = False,
):
    """List files in a directory."""
    target = _safe_path(path)
    if target is None:
        return {
            "error": "path outside workspace"
        }
    if not target.exists():
        return {"error": f"not found: {path}"}
    if not target.is_dir():
        return {"error": "not a directory"}

    files = []
    if recursive:
        for f in sorted(target.rglob("*")):
            if f.is_file():
                rel = f.relative_to(WORKSPACE)
                files.append(str(rel))
    else:
        for f in sorted(target.iterdir()):
            rel = f.relative_to(WORKSPACE)
            suffix = "/" if f.is_dir() else ""
            files.append(str(rel) + suffix)

    return {"path": path, "files": files}
