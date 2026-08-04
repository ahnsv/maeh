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


# Fixed attribute schema for `maeh list` rows / --filter (independent of contents).
LIST_KEYS = ("id", "status", "todo", "running", "done", "failed")


def _plan_row(tree: PlanTree) -> dict:
    nodes = list(tree.walk())
    row = {"id": tree.root.id, "status": tree.root.status.value}
    for st in (Status.TODO, Status.RUNNING, Status.DONE, Status.FAILED):
        row[st.value] = sum(1 for n in nodes if n.status is st)
    return row


def list_plans(home: Path) -> list[dict]:
    plans = home / "plans"
    if not plans.is_dir():
        return []
    return [_plan_row(load_plan(p.stem, home)) for p in sorted(plans.glob("*.json"))]


def filter_plans(rows: list[dict], filters: dict[str, str]) -> list[dict]:
    for key in filters:
        if key not in LIST_KEYS:
            raise KeyError(f"unknown filter key {key!r}; valid: {', '.join(LIST_KEYS)}")
    return [r for r in rows if all(str(r[k]) == v for k, v in filters.items())]


def save_handle(home: Path, handle: dict) -> Path:
    """Record an opened workspace (node_id, backend, ref, worktree) so worktrees
    are recoverable — a future gc/reconcile reads these instead of scanning."""
    require_safe_segment(handle["node_id"])
    d = private_subdir(home, "workspaces")
    path = d / f"{handle['node_id']}.json"
    write_private(path, json.dumps(handle, ensure_ascii=False, indent=2))
    return path
