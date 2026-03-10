from typing import List, Dict


class ShortTermMemory:
    """In-RAM sliding-window message buffer."""

    def __init__(self, max_messages: int = 30):
        self.max_messages = max_messages
        self._messages: List[Dict[str, str]] = []

    def add(self, role: str, content: str):
        self._messages.append({"role": role, "content": content})
        if len(self._messages) > self.max_messages:
            self._messages = self._messages[-self.max_messages :]

    @property
    def messages(self) -> List[Dict[str, str]]:
        return list(self._messages)
