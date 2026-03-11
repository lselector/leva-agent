"""FastAPI app for the LLM Gateway."""

from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from ..config import BASE_DIR
from .routes_chat import router as chat_router
from .routes_models import router as models_router

app = FastAPI(title="Jarvis LLM Gateway")

# Register API routers
app.include_router(chat_router)
app.include_router(models_router)


# --------------------------------------------------------------
@app.get("/api/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "server": "llm"}


# --------------------------------------------------------------
# Mount frontend static files LAST
# (after all /api/ routes are registered)
app.mount(
    "/",
    StaticFiles(
        directory=str(BASE_DIR / "frontend"),
        html=True,
    ),
    name="frontend",
)
