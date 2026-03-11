"""SSE streaming helpers with tool-call loop."""

import json
from typing import List
from ..models import _client, chat
from ..config import MODEL_NAME
from .tool_dispatch import (
    get_schemas,
    execute_tool_call,
)

MAX_TOOL_ROUNDS = 10


# --------------------------------------------------------------
async def stream_chat(messages: List[dict]):
    """Yield SSE data lines with tool-call loop."""
    tools_schema = get_schemas()
    full_text = ""

    for _round in range(MAX_TOOL_ROUNDS):
        stream = _client.chat.completions.create(
            model=MODEL_NAME,
            messages=messages,
            tools=tools_schema,
            stream=True,
        )

        # Collect streamed response
        content_buf = ""
        tool_calls_buf = {}
        finish_reason = None

        for chunk in stream:
            choice = chunk.choices[0]
            delta = choice.delta
            finish_reason = (
                choice.finish_reason
                or finish_reason
            )

            # Stream content tokens
            if delta.content:
                token = delta.content
                content_buf += token
                full_text += token
                yield (
                    "data: "
                    + json.dumps(
                        {"token": token}
                    )
                    + "\n\n"
                )

            # Accumulate tool call chunks
            if delta.tool_calls:
                for tc in delta.tool_calls:
                    idx = tc.index
                    if idx not in tool_calls_buf:
                        tool_calls_buf[idx] = {
                            "id": tc.id or "",
                            "name": "",
                            "args": "",
                        }
                    entry = tool_calls_buf[idx]
                    if tc.id:
                        entry["id"] = tc.id
                    if tc.function:
                        if tc.function.name:
                            entry["name"] += (
                                tc.function.name
                            )
                        if tc.function.arguments:
                            entry["args"] += (
                                tc.function
                                .arguments
                            )

        # If no tool calls, we're done
        if finish_reason != "tool_calls":
            break

        # Execute tool calls
        yield _status_event(
            "Using tools..."
        )

        # Build assistant message with tool_calls
        tc_list = _build_tool_calls(
            tool_calls_buf
        )
        messages.append({
            "role": "assistant",
            "content": content_buf or None,
            "tool_calls": tc_list,
        })

        # Execute each tool and add results
        for tc in tc_list:
            result = _execute_tc(tc)
            messages.append({
                "role": "tool",
                "tool_call_id": tc["id"],
                "content": result,
            })

        # Loop back to get LLM's final answer

    yield "data: [DONE]\n\n"
    yield (
        "data: "
        + json.dumps({"full_text": full_text})
        + "\n\n"
    )


# --------------------------------------------------------------
def _status_event(msg: str) -> str:
    """Create a status SSE event."""
    return (
        "data: "
        + json.dumps({"status": msg})
        + "\n\n"
    )


# --------------------------------------------------------------
def _build_tool_calls(buf: dict) -> list:
    """Convert accumulated chunks to tool_calls."""
    result = []
    for idx in sorted(buf.keys()):
        entry = buf[idx]
        result.append({
            "id": entry["id"],
            "type": "function",
            "function": {
                "name": entry["name"],
                "arguments": entry["args"],
            },
        })
    return result


# --------------------------------------------------------------
def _execute_tc(tc: dict) -> str:
    """Execute a tool call dict."""
    name = tc["function"]["name"]
    args_str = tc["function"]["arguments"]
    try:
        args = json.loads(args_str)
    except json.JSONDecodeError:
        return "Error: invalid JSON args"

    from ..tools.registry import TOOLS
    fn = TOOLS.get(name)
    if fn is None:
        return f"Error: unknown tool '{name}'"
    try:
        return str(fn(**args))
    except Exception as e:
        return f"Error running {name}: {e}"
