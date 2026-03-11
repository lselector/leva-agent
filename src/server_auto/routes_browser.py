"""Browser automation endpoints."""

from fastapi import APIRouter
from fastapi.responses import Response
from pydantic import BaseModel
from typing import Optional
from .browser_manager import browser_mgr

router = APIRouter(prefix="/browser")


# --------------------------------------------------------------
class NavigateRequest(BaseModel):
    """Request to navigate to a URL."""
    url: str


# --------------------------------------------------------------
class ClickRequest(BaseModel):
    """Request to click an element."""
    selector: str


# --------------------------------------------------------------
class TypeRequest(BaseModel):
    """Request to type into an element."""
    selector: str
    text: str


# --------------------------------------------------------------
class ExtractRequest(BaseModel):
    """Request to extract text from URL."""
    url: str


# --------------------------------------------------------------
class ScreenshotRequest(BaseModel):
    """Request to screenshot a URL."""
    url: str


# --------------------------------------------------------------
@router.post("/navigate")
async def navigate(req: NavigateRequest):
    """Navigate to a URL."""
    try:
        result = await browser_mgr.navigate(
            req.url
        )
        return result
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/extract")
async def extract(req: ExtractRequest):
    """Extract text content from a URL."""
    try:
        result = await browser_mgr.extract_text(
            req.url
        )
        return result
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/screenshot")
async def screenshot(req: ScreenshotRequest):
    """Take a screenshot of a URL."""
    try:
        data = await browser_mgr.screenshot(
            req.url
        )
        return Response(
            content=data,
            media_type="image/png",
        )
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/click")
async def click(req: ClickRequest):
    """Click an element on current page."""
    try:
        result = await browser_mgr.click_element(
            req.selector
        )
        return result
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/type")
async def type_text(req: TypeRequest):
    """Type text into an element."""
    try:
        result = await browser_mgr.type_text(
            req.selector, req.text
        )
        return result
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.get("/content")
async def get_content():
    """Get current page HTML content."""
    try:
        result = (
            await browser_mgr.get_page_content()
        )
        return result
    except Exception as e:
        return {"error": str(e)}


# --------------------------------------------------------------
@router.post("/close")
async def close_browser():
    """Close the browser."""
    try:
        await browser_mgr.close()
        return {"status": "ok"}
    except Exception as e:
        return {"error": str(e)}


# ==============================================================
# Gmail endpoints
# ==============================================================

from . import gmail_actions
from . import linkedin_actions


# --------------------------------------------------------------
class EmailRequest(BaseModel):
    """Request to compose an email."""
    to: str
    subject: str
    body: str


# --------------------------------------------------------------
class LikeRequest(BaseModel):
    """Request to like a LinkedIn post."""
    post_index: int = 0
    dry_run: bool = True


# --------------------------------------------------------------
@router.post("/gmail/inbox")
async def gmail_inbox():
    """Get recent emails from Gmail."""
    return await gmail_actions.get_inbox()


# --------------------------------------------------------------
@router.post("/gmail/compose")
async def gmail_compose(req: EmailRequest):
    """Compose an email draft."""
    return await gmail_actions.send_email(
        req.to, req.subject, req.body
    )


# --------------------------------------------------------------
@router.post("/gmail/send-confirm")
async def gmail_send_confirm():
    """Confirm and send the composed email."""
    return await gmail_actions.confirm_send()


# ==============================================================
# LinkedIn endpoints
# ==============================================================


# --------------------------------------------------------------
@router.post("/linkedin/feed")
async def linkedin_feed():
    """Get recent LinkedIn feed posts."""
    return await linkedin_actions.get_feed()


# --------------------------------------------------------------
@router.post("/linkedin/like")
async def linkedin_like(req: LikeRequest):
    """Like a LinkedIn post."""
    return await linkedin_actions.like_post(
        req.post_index, req.dry_run
    )
