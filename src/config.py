"""Configuration and paths for mini-claw."""

from pathlib import Path
from dotenv import load_dotenv
import os

load_dotenv()

BASE_DIR = Path(__file__).resolve().parents[1]
PROMPTS_DIR = BASE_DIR / "prompts"
SOUL_DIR = BASE_DIR / "soul"
MEMORY_DIR = BASE_DIR / "memory"
MEMORY_TOPICS_DIR = MEMORY_DIR / "topics"
REFERENCE_DIR = BASE_DIR / "reference"

MODEL_NAME = os.getenv(
    "MODEL_NAME", "gpt-4.1-mini"
)
OPENAI_API_KEY = os.getenv(
    "OPENAI_API_KEY", ""
)

LLM_PORT = int(os.getenv("LLM_PORT", "8000"))
AUTO_PORT = int(os.getenv("AUTO_PORT", "8001"))
AUTO_BASE_URL = (
    f"http://127.0.0.1:{AUTO_PORT}"
)
