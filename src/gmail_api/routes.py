"""FastAPI routes for Gmail API operations."""
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from .client import (
    get_email,
    get_inbox,
    get_labels,
    search_emails,
    send_email,
)

router = APIRouter(prefix="/api/gmail", tags=["gmail"])


class InboxRequest(BaseModel):
    max_results: int = 15


class SendRequest(BaseModel):
    to: str
    subject: str
    body: str


class SearchRequest(BaseModel):
    query: str
    max_results: int = 10


# ---------------------------------------------------------------

@router.post("/inbox")
async def gmail_inbox(req: InboxRequest):
    """Get recent inbox emails."""
    try:
        return get_inbox(max_results=req.max_results)
    except FileNotFoundError as e:
        raise HTTPException(status_code=503, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/email/{msg_id}")
async def gmail_get_email(msg_id: str):
    """Get full email by message ID."""
    try:
        return get_email(msg_id)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/send")
async def gmail_send(req: SendRequest):
    """Send an email."""
    try:
        return send_email(
            to=req.to,
            subject=req.subject,
            body=req.body,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/search")
async def gmail_search(req: SearchRequest):
    """Search emails using Gmail query syntax."""
    try:
        return search_emails(
            query=req.query,
            max_results=req.max_results,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/labels")
async def gmail_labels():
    """List all Gmail labels."""
    try:
        return get_labels()
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
