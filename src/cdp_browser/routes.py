"""FastAPI routes for CDP browser and LinkedIn operations."""
from fastapi import APIRouter, HTTPException
from fastapi.responses import Response
from pydantic import BaseModel

from .actions import (
    linkedin_feed,
    linkedin_like,
    web_fetch,
    web_screenshot,
    web_search,
)
from .client import list_tabs

router = APIRouter(tags=["browser"])


class SearchRequest(BaseModel):
    query: str


class FetchRequest(BaseModel):
    url: str


class LikeRequest(BaseModel):
    post_index: int = 0
    dry_run: bool = True


# ---------------------------------------------------------------
# Web endpoints

@router.get("/api/web/tabs")
async def web_tabs():
    """List open Chrome tabs."""
    try:
        return list_tabs()
    except Exception as e:
        raise HTTPException(
            status_code=503,
            detail=f"Chrome CDP not available: {e}",
        )


@router.post("/api/web/search")
async def web_search_endpoint(req: SearchRequest):
    """Search Google and return results."""
    try:
        return web_search(req.query)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/api/web/fetch")
async def web_fetch_endpoint(req: FetchRequest):
    """Fetch a URL and return its text content."""
    try:
        content = web_fetch(req.url)
        return {"url": req.url, "content": content}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/api/web/screenshot")
async def web_screenshot_endpoint(req: FetchRequest):
    """Take a screenshot of a URL and return PNG bytes."""
    try:
        png = web_screenshot(req.url)
        return Response(content=png, media_type="image/png")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# ---------------------------------------------------------------
# LinkedIn endpoints

@router.post("/api/linkedin/feed")
async def linkedin_feed_endpoint():
    """Get LinkedIn feed posts."""
    try:
        return linkedin_feed()
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/api/linkedin/like")
async def linkedin_like_endpoint(req: LikeRequest):
    """Like a LinkedIn feed post (dry_run=True by default)."""
    try:
        return linkedin_like(
            post_index=req.post_index,
            dry_run=req.dry_run,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
