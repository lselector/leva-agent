from typing import Protocol, Any


class Tool(Protocol):
    name: str
    description: str

    def __call__(self, **kwargs) -> Any:
        ...
