"""LLM wrapper for OpenAI chat completions."""

from openai import OpenAI
from .config import OPENAI_API_KEY, MODEL_NAME

_client = OpenAI(api_key=OPENAI_API_KEY)


# --------------------------------------------------------------
def chat(messages, tools_schema=None):
    """Send messages to the LLM and return response."""
    kwargs = {}
    if tools_schema:
        kwargs["tools"] = tools_schema
    resp = _client.chat.completions.create(
        model=MODEL_NAME,
        messages=messages,
        **kwargs,
    )
    return resp
