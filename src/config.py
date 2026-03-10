"""Configuration and paths for mini-claw."""

from pathlib import Path
from dotenv import load_dotenv
import os

load_dotenv()

BASE_DIR = Path(__file__).resolve().parents[1]
PROMPTS_DIR = BASE_DIR / "prompts"
MEMORY_DIR = BASE_DIR / "memory"

MODEL_NAME = os.getenv(
    "MODEL_NAME", "gpt-4.1-mini"
)
OPENAI_API_KEY = os.getenv(
    "OPENAI_API_KEY", ""
)
