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


