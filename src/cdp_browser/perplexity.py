"""Perplexity API fallback for web research.

Used when CDP browser is unavailable or unreliable.
Requires PERPLEXITY_API_KEY in .env
"""
import os
import httpx
from dotenv import load_dotenv

load_dotenv()

PPLX_URL = "https://api.perplexity.ai/chat/completions"
PPLX_MODEL = "sonar"


def web_research(query: str) -> str:
    """Search the web via Perplexity API.

    Returns a web-grounded answer as plain text.
    No browser required — uses PERPLEXITY_API_KEY from .env.
    """
    api_key = os.getenv("PERPLEXITY_API_KEY")
    if not api_key:
        raise EnvironmentError(
            "PERPLEXITY_API_KEY not set in .env"
        )

    resp = httpx.post(
        PPLX_URL,
        headers={"Authorization": f"Bearer {api_key}"},
        json={
            "model": PPLX_MODEL,
            "messages": [
                {"role": "user", "content": query}
            ],
        },
        timeout=30,
    )
    resp.raise_for_status()
    data = resp.json()
    return (
        data["choices"][0]["message"]["content"]
    )
