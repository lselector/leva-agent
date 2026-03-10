"""Main agent loop with tool-call handling."""

import json
from typing import Dict, Any
from .memory_store.short_term import ShortTermMemory
from .tools.registry import TOOLS, get_tools_schema
from .models import chat
from .config import PROMPTS_DIR


# --------------------------------------------------------------
def load_system_prompt() -> str:
    """Load the system prompt from SYSTEM.md."""
    return (
        PROMPTS_DIR / "SYSTEM.md"
    ).read_text(encoding="utf-8")


# --------------------------------------------------------------
def build_base_messages(stm: ShortTermMemory):
    """Build message list with system prompt + history."""
    system = {
        "role": "system",
        "content": load_system_prompt(),
    }
    return [system, *stm.messages]


# --------------------------------------------------------------
def _parse_tool_args(raw_args):
    """Parse tool arguments from string or dict."""
    if isinstance(raw_args, str):
        return json.loads(raw_args)
    return raw_args or {}


# --------------------------------------------------------------
def _execute_tool(name, args):
    """Execute a tool by name with given args."""
    try:
        return TOOLS[name](**args)
    except Exception as e:
        return f"Error: {e}"


# --------------------------------------------------------------
def _handle_tool_call(choice, stm):
    """Handle a tool call and return final answer."""
    tool_call = choice.message.tool_calls[0]
    name = tool_call.function.name
    args = _parse_tool_args(
        tool_call.function.arguments
    )
    result = _execute_tool(name, args)
    stm.add(
        "assistant", f"[tool:{name}] {result}"
    )
    messages = build_base_messages(stm)
    resp2 = chat(messages)
    answer = (
        resp2.choices[0].message.content or ""
    )
    stm.add("assistant", answer)
    return answer


# --------------------------------------------------------------
def run_agent(
    user_input: str, stm: ShortTermMemory
) -> str:
    """Run one turn of the agent loop."""
    stm.add("user", user_input)
    messages = build_base_messages(stm)
    tools_schema = get_tools_schema()
    resp = chat(messages, tools_schema)
    choice = resp.choices[0]

    if choice.finish_reason == "tool_calls":
        return _handle_tool_call(choice, stm)

    answer = choice.message.content or ""
    stm.add("assistant", answer)
    return answer
