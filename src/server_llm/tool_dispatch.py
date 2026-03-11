"""Dispatch tool calls from the LLM."""

import json
from ..tools.registry import TOOLS, get_tools_schema


# --------------------------------------------------------------
def get_schemas():
    """Return all tool schemas for OpenAI."""
    return get_tools_schema()


# --------------------------------------------------------------
def execute_tool_call(tool_call) -> str:
    """Execute a single tool call, return result."""
    name = tool_call.function.name
    try:
        args = json.loads(
            tool_call.function.arguments
        )
    except json.JSONDecodeError:
        return f"Error: invalid JSON args"

    fn = TOOLS.get(name)
    if fn is None:
        return f"Error: unknown tool '{name}'"

    try:
        result = fn(**args)
        return str(result)
    except Exception as e:
        return f"Error running {name}: {e}"
