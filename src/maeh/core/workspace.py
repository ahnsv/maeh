from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from maeh.core.models import Node

Runner = Callable[[list[str]], str]


@dataclass(frozen=True)
class WorkspaceHandle:
    node_id: str
    backend: str
    ref: str


def _run(cmd: list[str]) -> str:
    return subprocess.run(cmd, check=True, capture_output=True, text=True).stdout


def _label(node: Node) -> str:
    return f"maeh-{node.id}"


def _open_tmux(node: Node, cwd: Path, run: Runner) -> WorkspaceHandle:
    session = _label(node)
    # -A = attach-or-create, so re-opening the same node is idempotent.
    run(["tmux", "new-session", "-A", "-d", "-s", session, "-c", str(cwd)])
    return WorkspaceHandle(node.id, "tmux", session)


def _open_herdr(node: Node, cwd: Path, run: Runner) -> WorkspaceHandle:
    label = _label(node)
    # herdr has no attach-or-create; get idempotency by finding an existing
    # workspace with our label before creating a new one.
    workspaces = json.loads(run(["herdr", "workspace", "list"]))["result"]["workspaces"]
    for ws in workspaces:
        if ws.get("label") == label:
            return WorkspaceHandle(node.id, "herdr", ws["workspace_id"])
    created = json.loads(
        run(
            [
                "herdr",
                "workspace",
                "create",
                "--cwd",
                str(cwd),
                "--label",
                label,
                "--no-focus",
            ]
        )
    )
    return WorkspaceHandle(
        node.id, "herdr", created["result"]["workspace"]["workspace_id"]
    )


_BACKENDS: dict[str, Callable[[Node, Path, Runner], WorkspaceHandle]] = {
    "tmux": _open_tmux,
    "herdr": _open_herdr,
}

SUPPORTED_BACKENDS = frozenset(_BACKENDS)


def open_workspace(
    node: Node, cwd: Path, backend: str = "tmux", runner: Runner = _run
) -> WorkspaceHandle:
    try:
        opener = _BACKENDS[backend]
    except KeyError:
        raise ValueError(
            f"unknown backend {backend!r}; available: {sorted(SUPPORTED_BACKENDS)}"
        ) from None
    try:
        return opener(node, cwd, runner)
    except FileNotFoundError as e:
        raise RuntimeError(
            f"backend {backend!r} binary not found — is it installed?"
        ) from e
    except subprocess.CalledProcessError as e:
        detail = e.stderr or e
        raise RuntimeError(
            f"backend {backend!r} command failed (daemon running?): {detail}"
        ) from e
