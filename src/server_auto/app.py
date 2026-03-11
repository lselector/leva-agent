"""FastAPI app for the Automation Engine."""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from .routes_files import router as files_router
from .routes_jobs import router as jobs_router
from .routes_browser import router as browser_router
from src.gmail_api.routes import router as gmail_router
from src.cdp_browser.routes import router as cdp_router

app = FastAPI(title="Jarvis Automation Engine")

# Allow cross-origin from Server A
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:8000",
        "http://127.0.0.1:8000",
    ],
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register routers
app.include_router(files_router)
app.include_router(jobs_router)
app.include_router(browser_router)
app.include_router(gmail_router)
app.include_router(cdp_router)


# --------------------------------------------------------------
@app.get("/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "server": "auto"}
