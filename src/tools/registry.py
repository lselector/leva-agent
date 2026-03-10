"""Map tool name to function + OpenAI schemas."""

from typing import Dict, Callable, Any
from . import files, memory_tools

ToolFn = Callable[..., Any]

TOOLS: Dict[str, ToolFn] = {
    "file_read": files.file_read,
    "file_write": files.file_write,
    "memory_append": memory_tools.memory_append,
    "memory_search": memory_tools.memory_search,
    "soul_read": memory_tools.soul_read,
    "memory_topic_write": (
        memory_tools.memory_topic_write
    ),
    "memory_topic_read": (
        memory_tools.memory_topic_read
    ),
    "memory_topic_list": (
        memory_tools.memory_topic_list
    ),
    "reference_read": (
        memory_tools.reference_read
    ),
    "reference_write": (
        memory_tools.reference_write
    ),
    "reference_list": (
        memory_tools.reference_list
    ),
    "reference_search": (
        memory_tools.reference_search
    ),
}


# --------------------------------------------------------------
def _file_read_schema():
    """Schema for file_read tool."""
    return {
        "type": "function",
        "function": {
            "name": "file_read",
            "description": (
                "Read a small text file "
                "from the workspace."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": (
                            "Relative path to "
                            "the file."
                        ),
                    }
                },
                "required": ["path"],
            },
        },
    }


# --------------------------------------------------------------
def _file_write_schema():
    """Schema for file_write tool."""
    return {
        "type": "function",
        "function": {
            "name": "file_write",
            "description": (
                "Overwrite or create a "
                "text file."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": (
                            "Relative path to "
                            "the file."
                        ),
                    },
                    "content": {
                        "type": "string",
                        "description": (
                            "The text content "
                            "to write."
                        ),
                    },
                },
                "required": [
                    "path", "content"
                ],
            },
        },
    }


# --------------------------------------------------------------
def _memory_append_schema():
    """Schema for memory_append tool."""
    return {
        "type": "function",
        "function": {
            "name": "memory_append",
            "description": (
                "Append a note to today's "
                "daily memory file."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": (
                            "The note text "
                            "to append."
                        ),
                    }
                },
                "required": ["text"],
            },
        },
    }


# --------------------------------------------------------------
def _memory_search_schema():
    """Schema for memory_search tool."""
    return {
        "type": "function",
        "function": {
            "name": "memory_search",
            "description": (
                "Search daily and topic "
                "memory files for a query."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": (
                            "Search string to "
                            "match against "
                            "memory entries."
                        ),
                    }
                },
                "required": ["query"],
            },
        },
    }


# --------------------------------------------------------------
def _soul_read_schema():
    """Schema for soul_read tool."""
    return {
        "type": "function",
        "function": {
            "name": "soul_read",
            "description": (
                "Read all Layer 1 soul "
                "files (identity, agents, "
                "user profile)."
            ),
            "parameters": {
                "type": "object",
                "properties": {},
                "required": [],
            },
        },
    }


# --------------------------------------------------------------
def _memory_topic_write_schema():
    """Schema for memory_topic_write tool."""
    return {
        "type": "function",
        "function": {
            "name": "memory_topic_write",
            "description": (
                "Write or overwrite a topic "
                "summary in Layer 2 memory."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": (
                            "Topic name "
                            "(becomes filename)."
                        ),
                    },
                    "content": {
                        "type": "string",
                        "description": (
                            "Markdown content "
                            "for the topic."
                        ),
                    },
                },
                "required": [
                    "topic", "content"
                ],
            },
        },
    }


# --------------------------------------------------------------
def _memory_topic_read_schema():
    """Schema for memory_topic_read tool."""
    return {
        "type": "function",
        "function": {
            "name": "memory_topic_read",
            "description": (
                "Read a specific topic "
                "file from Layer 2 memory."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": (
                            "Topic name to read."
                        ),
                    }
                },
                "required": ["topic"],
            },
        },
    }


# --------------------------------------------------------------
def _memory_topic_list_schema():
    """Schema for memory_topic_list tool."""
    return {
        "type": "function",
        "function": {
            "name": "memory_topic_list",
            "description": (
                "List all topic files "
                "in Layer 2 memory."
            ),
            "parameters": {
                "type": "object",
                "properties": {},
                "required": [],
            },
        },
    }


# --------------------------------------------------------------
def _reference_read_schema():
    """Schema for reference_read tool."""
    return {
        "type": "function",
        "function": {
            "name": "reference_read",
            "description": (
                "Read a document from "
                "the Layer 3 reference "
                "library."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": (
                            "Document name "
                            "to read."
                        ),
                    }
                },
                "required": ["name"],
            },
        },
    }


# --------------------------------------------------------------
def _reference_write_schema():
    """Schema for reference_write tool."""
    return {
        "type": "function",
        "function": {
            "name": "reference_write",
            "description": (
                "Write or overwrite a "
                "document in the Layer 3 "
                "reference library."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": (
                            "Document name "
                            "(becomes filename)."
                        ),
                    },
                    "content": {
                        "type": "string",
                        "description": (
                            "Full document "
                            "content."
                        ),
                    },
                },
                "required": [
                    "name", "content"
                ],
            },
        },
    }


# --------------------------------------------------------------
def _reference_list_schema():
    """Schema for reference_list tool."""
    return {
        "type": "function",
        "function": {
            "name": "reference_list",
            "description": (
                "List all documents in "
                "the Layer 3 reference "
                "library."
            ),
            "parameters": {
                "type": "object",
                "properties": {},
                "required": [],
            },
        },
    }


# --------------------------------------------------------------
def _reference_search_schema():
    """Schema for reference_search tool."""
    return {
        "type": "function",
        "function": {
            "name": "reference_search",
            "description": (
                "Search lines in Layer 3 "
                "reference documents."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": (
                            "Search string to "
                            "match against "
                            "reference docs."
                        ),
                    }
                },
                "required": ["query"],
            },
        },
    }


# --------------------------------------------------------------
def get_tools_schema():
    """Return OpenAI-style tool schemas."""
    return [
        _file_read_schema(),
        _file_write_schema(),
        _memory_append_schema(),
        _memory_search_schema(),
        _soul_read_schema(),
        _memory_topic_write_schema(),
        _memory_topic_read_schema(),
        _memory_topic_list_schema(),
        _reference_read_schema(),
        _reference_write_schema(),
        _reference_list_schema(),
        _reference_search_schema(),
    ]
