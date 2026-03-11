"""Chat endpoint for the LLM Gateway."""

from fastapi import APIRouter, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from typing import Optional
from ..models import chat
from ..tools.memory_tools import soul_read
from ..config import PROMPTS_DIR
from .session_store import store
from .streaming import stream_chat

router = APIRouter(prefix="/api")


# --------------------------------------------------------------
class ChatRequest(BaseModel):
    """Incoming chat request."""
    message: str
    content: Optional[object] = None
    session_id: Optional[str] = None
    stream: Optional[bool] = True


# --------------------------------------------------------------
class ChatResponse(BaseModel):
    """Chat response."""
    reply: str
    session_id: Optional[str] = None


# --------------------------------------------------------------
def _load_system_prompt() -> str:
    """Load system prompt with soul context."""
    base = (
        PROMPTS_DIR / "SYSTEM.md"
    ).read_text(encoding="utf-8")
    soul = soul_read()
    return (
        f"{base}\n\n"
        f"# Core Identity (Soul)\n\n"
        f"{soul}"
    )


# --------------------------------------------------------------
def _build_messages(sid, system_prompt):
    """Build message list from session history."""
    history = store.get_messages(sid)
    return [
        {
            "role": "system",
            "content": system_prompt,
        },
        *history,
    ]


# --------------------------------------------------------------
@router.post("/chat")
async def chat_endpoint(req: ChatRequest):
    """Handle chat — streaming or non-streaming."""
    sid = req.session_id or "default"
    system_prompt = _load_system_prompt()

    # Determine user content (multipart or text)
    user_content = req.content or req.message

    # Store text-only version for session log
    store.add_message(sid, "user", req.message)

    # Build messages, replace last user msg
    # with rich content if files attached
    messages = _build_messages(
        sid, system_prompt
    )
    if req.content and isinstance(
        req.content, list
    ):
        # Replace last user message content
        # with multipart content array
        messages[-1] = {
            "role": "user",
            "content": req.content,
        }

    if req.stream:
        return _stream_response(sid, messages)
    return _sync_response(sid, messages)


# --------------------------------------------------------------
def _stream_response(sid, messages):
    """Return SSE streaming response."""
    collected = {"text": ""}

    async def generate():
        """Yield SSE events and save result."""
        async for line in stream_chat(messages):
            # Capture full_text event
            if '"full_text"' in line:
                import json
                data = line.replace(
                    "data: ", ""
                ).strip()
                obj = json.loads(data)
                collected["text"] = (
                    obj["full_text"]
                )
                store.add_message(
                    sid,
                    "assistant",
                    collected["text"],
                )
            yield line

    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "X-Accel-Buffering": "no",
        },
    )


# --------------------------------------------------------------
def _sync_response(sid, messages):
    """Return non-streaming JSON response."""
    resp = chat(messages)
    answer = (
        resp.choices[0].message.content or ""
    )
    store.add_message(sid, "assistant", answer)
    return ChatResponse(
        reply=answer,
        session_id=sid,
    )


# --------------------------------------------------------------
@router.get("/chat/history")
async def list_sessions():
    """List all session IDs."""
    return store.list_sessions()


# --------------------------------------------------------------
@router.get("/chat/{session_id}")
async def get_session(session_id: str):
    """Get messages for a session."""
    msgs = store.get_session(session_id)
    if msgs is None:
        raise HTTPException(
            status_code=404,
            detail="Session not found",
        )
    return msgs


# --------------------------------------------------------------
@router.delete("/chat/{session_id}")
async def delete_session(session_id: str):
    """Delete a session."""
    deleted = store.delete_session(session_id)
    if not deleted:
        raise HTTPException(
            status_code=404,
            detail="Session not found",
        )
    return {"status": "deleted"}
