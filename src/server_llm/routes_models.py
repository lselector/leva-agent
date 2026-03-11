"""Model selection endpoints."""

from fastapi import APIRouter
from pydantic import BaseModel
from .. import config

router = APIRouter(prefix="/api")

AVAILABLE_MODELS = [
    "gpt-4.1-mini",
    "gpt-4.1",
    "gpt-4o",
    "gpt-4o-mini",
    "o3-mini",
]


# --------------------------------------------------------------
class ModelSwitch(BaseModel):
    """Request to switch model."""
    model: str


# --------------------------------------------------------------
@router.get("/models")
async def get_models():
    """Return current and available models."""
    return {
        "current": config.MODEL_NAME,
        "available": AVAILABLE_MODELS,
    }


# --------------------------------------------------------------
@router.put("/models/current")
async def set_model(req: ModelSwitch):
    """Switch the active model."""
    if req.model not in AVAILABLE_MODELS:
        return {
            "error": (
                f"Unknown model: {req.model}"
            ),
            "available": AVAILABLE_MODELS,
        }
    config.MODEL_NAME = req.model
    return {
        "status": "ok",
        "model": config.MODEL_NAME,
    }
