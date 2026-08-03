from __future__ import annotations

import fcntl
import json
import os
from collections.abc import Callable
from contextlib import contextmanager
from pathlib import Path

from maeh.core.fsutil import private_subdir, write_private
from maeh.core.models import Node, PlanTree, Status, require_safe_segment


def _to_dict(node: Node) -> dict:
    return {
        "id": node.id,
        "name": node.name,
        "status": node.status.value,
        "path": node.path,
        "children": [_to_dict(c) for c in node.children],
    }


def _from_dict(d: dict) -> Node:
    return Node(
        id=d["id"],
        name=d["name"],
        status=Status(d["status"]),
        path=d.get("path"),
        children=[_from_dict(c) for c in d["children"]],
    )


def plan_to_dict(tree: PlanTree) -> dict:
    return _to_dict(tree.root)


def save_plan(tree: PlanTree, home: Path) -> Path:
    plans = private_subdir(home, "plans")
    path = plans / f"{tree.root.id}.json"  # root.id validated in __post_init__
    write_private(path, json.dumps(_to_dict(tree.root), ensure_ascii=False, indent=2))
    return path


def load_plan(plan_id: str, home: Path) -> PlanTree:
    require_safe_segment(plan_id)
    path = home / "plans" / f"{plan_id}.json"
    return PlanTree(_from_dict(json.loads(path.read_text())))


@contextmanager
def _plan_lock(home: Path, plan_id: str):
    require_safe_segment(plan_id)
    plans = private_subdir(home, "plans")
    fd = os.open(plans / f"{plan_id}.lock", os.O_WRONLY | os.O_CREAT, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


def update_plan(
    home: Path, plan_id: str, mutate: Callable[[PlanTree], None]
) -> PlanTree:
    """The single writer: lock, load, mutate, atomically save. All plan
    mutations route through here so concurrent primary/critic can't lose updates."""
    with _plan_lock(home, plan_id):
        tree = load_plan(plan_id, home)
        mutate(tree)
        save_plan(tree, home)
        return tree
