from __future__ import annotations

import re
from collections.abc import Iterator
from dataclasses import dataclass, field
from enum import StrEnum

# Must start and end alphanumeric; dots/dashes/underscores allowed inside.
# Rejects "..", leading/trailing dots, slashes — safe as a filename segment.
_SAFE_SEGMENT = re.compile(r"^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9])?$")


def require_safe_segment(value: str) -> str:
    if not _SAFE_SEGMENT.fullmatch(value):
        raise ValueError(f"unsafe identifier: {value!r}")
    return value


class Status(StrEnum):
    TODO = "todo"
    RUNNING = "running"
    DONE = "done"
    FAILED = "failed"


@dataclass
class Node:
    id: str
    name: str
    status: Status = Status.TODO
    path: str | None = None  # code location the node's workspace opens in
    brief: str | None = None  # frozen plan-time task detail; rendered into capsules
    children: list[Node] = field(default_factory=list)

    def __post_init__(self) -> None:
        require_safe_segment(self.id)


@dataclass
class Increment:
    node_id: str
    kind: str  # "pr" | "document" | "artifact"
    ref: str  # URL / path / id


@dataclass
class PlanTree:
    root: Node

    def walk(self) -> Iterator[Node]:
        stack = [self.root]
        while stack:
            node = stack.pop()
            yield node
            stack.extend(reversed(node.children))  # pre-order, left-to-right

    def find(self, node_id: str) -> Node | None:
        return next((n for n in self.walk() if n.id == node_id), None)
