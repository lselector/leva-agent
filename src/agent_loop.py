import json
from typing import Dict, Any
from .memory_store.short_term import ShortTermMemory
from .tools.registry import TOOLS, get_tools_schema
from .models import chat
from .config import PROMPTS_DIR


def load_system_prompt() -> str:
    return (PROMPTS_DIR / "SYSTEM.md").read_text(encoding="utf-8")


def build_base_messages(stm: ShortTermMemory):
    system = {"role": "system", "content": load_system_prompt()}
    return [system, *stm.messages]


def run_agent(user_input: str, stm: ShortTermMemory) -> str:
    stm.add("user", user_input)
    messages = build_base_messages(stm)

    tools_schema = get_tools_schema()
    resp = chat(messages, tools_schema)
    choice = resp.choices[0]

    # Handle tool calls (simple, single-step)
    if choice.finish_reason == "tool_calls":
        tool_call = choice.message.tool_calls[0]
        name = tool_call.function.name
        raw_args = tool_call.function.arguments
        if isinstance(raw_args, str):
            args: Dict[str, Any] = json.loads(raw_args)
        else:
            args = raw_args or {}

        try:
            result = TOOLS[name](**args)
        except Exception as e:
            result = f"Error: {e}"

        # Append the assistant message with tool_calls
        stm.add("assistant", f"[tool:{name}] {result}")

        # Follow-up call to let the model summarise the tool result
        messages = build_base_messages(stm)
        resp2 = chat(messages)
        answer = resp2.choices[0].message.content or ""
        stm.add("assistant", answer)
        return answer

    answer = choice.message.content or ""
    stm.add("assistant", answer)
    return answer
