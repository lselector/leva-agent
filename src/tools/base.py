"""Tool protocol interface."""

from typing import Protocol, Any


# --------------------------------------------------------------
class Tool(Protocol):
    """Protocol for tool implementations."""

    name: str
    description: str

    def __call__(self, **kwargs) -> Any:
        ...
