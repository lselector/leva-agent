from typing import Dict, Callable, Any
from . import files, memory_tools

ToolFn = Callable[..., Any]

TOOLS: Dict[str, ToolFn] = {
    "file_read": files.file_read,
    "file_write": files.file_write,
    "memory_append": memory_tools.memory_append,
    "memory_search": memory_tools.memory_search,
}


def get_tools_schema():
    """Return OpenAI-style tool schemas for all registered tools."""
    return [
        {
            "type": "function",
            "function": {
                "name": "file_read",
                "description": "Read a small text file from the workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to the file within the workspace.",
                        }
                    },
                    "required": ["path"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "file_write",
                "description": "Overwrite or create a text file in the workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path to the file within the workspace.",
                        },
                        "content": {
                            "type": "string",
                            "description": "The text content to write.",
                        },
                    },
                    "required": ["path", "content"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "memory_append",
                "description": "Append a note to today's memory markdown file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The note text to append.",
                        }
                    },
                    "required": ["text"],
                },
            },
        },
        {
            "type": "function",
            "function": {
                "name": "memory_search",
                "description": "Return lines from memory files that match the query.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search string to match against memory entries.",
                        }
                    },
                    "required": ["query"],
                },
            },
        },
    ]
