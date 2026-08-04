from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from maeh.core.models import Node

if TYPE_CHECKING:
    from maeh.core.config import Config

Runner = Callable[[list[str]], str]

_ROLE_CMD = {"editor": "editor_cmd", "primary": "primary_cmd", "critic": "critic_cmd"}


@dataclass(frozen=True)
class WorkspaceHandle:
    node_id: str
    backend: str
    ref: str
    worktree: str


def _run(cmd: list[str]) -> str:
    return subprocess.run(cmd, check=True, capture_output=True, text=True).stdout


def _label(node: Node) -> str:
    return f"maeh-{node.id}"


def _role_cmds(config: Config) -> list[str]:
    """Roles (from [workspace].panes) mapped to their [agents] command, skipping
    roles with no command. Node data is never mixed in — only operator config."""
    out: list[str] = []
    for role in config.workspace.panes_for(config.backend):
        attr = _ROLE_CMD.get(role)
        cmd = getattr(config.agents, attr, "") if attr else ""
        if cmd:
            out.append(cmd)
    return out


def resolve_worktree(
    prefix: str, location: str, repo: Path, node_id: str
) -> tuple[Path, str]:
    """Return (worktree_path, branch). Central when location is absolute/`~`
    (`<location>/<repo>/<branch>`), else project-local under the repo — a relative
    location is confined so it can't escape the repo. `prefix` is validated at
    config load; `node_id` is validated on the node, so `branch` is path-safe."""
    branch = f"{prefix}-{node_id}"
    if location.startswith("~") or Path(location).is_absolute():
        base = Path(location).expanduser() / repo.name
    else:
        base = (repo / location).resolve()
        if not base.is_relative_to(repo.resolve()):
            raise ValueError(f"worktree location {location!r} escapes the repo")
    return base / branch, branch


def _branch_exists(run: Runner, repo: Path, branch: str) -> bool:
    return bool(run(["git", "-C", str(repo), "branch", "--list", branch]).strip())


def _ensure_worktree(run: Runner, repo: Path, wt: Path, branch: str) -> None:
    if wt.exists():
        return  # idempotent by path
    run(["git", "-C", str(repo), "worktree", "prune"])
    add = ["git", "-C", str(repo), "worktree", "add", str(wt)]
    # branch-aware: check out an existing branch, else create it (`-b`).
    add += [branch] if _branch_exists(run, repo, branch) else ["-b", branch]
    run(add)


def _open_tmux(node: Node, config: Config, run: Runner) -> WorkspaceHandle:
    repo = Path(node.path).expanduser()
    wt, branch = resolve_worktree(
        config.worktree.prefix, config.worktree.location, repo, node.id
    )
    _ensure_worktree(run, repo, wt, branch)
    window = _label(node)
    run(["tmux", "new-session", "-A", "-d", "-s", "maeh", "-c", str(wt)])
    windows = run(
        ["tmux", "list-windows", "-t", "maeh", "-F", "#{window_name}"]
    ).split()
    if window not in windows:  # idempotent by window name — no duplicate panes
        run(["tmux", "new-window", "-t", "maeh", "-n", window, "-c", str(wt)])
        for i, cmd in enumerate(_role_cmds(config)):
            if i > 0:
                run(["tmux", "split-window", "-t", f"maeh:{window}", "-c", str(wt)])
            target = f"maeh:{window}.{i}"
            run(
                ["tmux", "send-keys", "-t", target, "-l", "--", cmd]
            )  # literal, no shell parse
            run(["tmux", "send-keys", "-t", target, "Enter"])
    return WorkspaceHandle(node.id, "tmux", window, str(wt))


def _herdr_pane_id(result: dict) -> str:
    pane = result.get("pane") or result.get("root_pane") or {}
    return pane.get("pane_id", "")


def _open_herdr(node: Node, config: Config, run: Runner) -> WorkspaceHandle:
    label = _label(node)
    workspaces = json.loads(run(["herdr", "workspace", "list"]))["result"]["workspaces"]
    for ws in workspaces:  # find-or-create by label — idempotent, no re-split
        if ws.get("label") == label:
            wt = (ws.get("worktree") or {}).get("checkout_path", "")
            return WorkspaceHandle(node.id, "herdr", ws["workspace_id"], wt)
    repo = Path(node.path).expanduser()
    branch = f"{config.worktree.prefix}-{node.id}"
    created = json.loads(
        run(
            [
                "herdr",
                "worktree",
                "create",
                "--cwd",
                str(repo),
                "--branch",
                branch,
                "--label",
                label,
            ]
        )
    )["result"]
    ws_id = created["workspace"]["workspace_id"]
    wt = (created["workspace"].get("worktree") or {}).get("checkout_path", "")
    pane = created["root_pane"]["pane_id"]
    for i, cmd in enumerate(_role_cmds(config)):
        if i > 0:
            # herdr: pane id is positional; direction is right|down.
            split = json.loads(
                run(["herdr", "pane", "split", pane, "--direction", "down"])
            )["result"]
            pane = _herdr_pane_id(split) or pane
        run(["herdr", "pane", "run", pane, cmd])
    return WorkspaceHandle(node.id, "herdr", ws_id, wt)


_BACKENDS: dict[str, Callable[[Node, Config, Runner], WorkspaceHandle]] = {
    "tmux": _open_tmux,
    "herdr": _open_herdr,
}

SUPPORTED_BACKENDS = frozenset(_BACKENDS)


def open_workspace(
    node: Node, config: Config, runner: Runner = _run
) -> WorkspaceHandle:
    if not node.path:
        raise ValueError(
            f"node {node.id!r} has no path; set one with `maeh plan add --path DIR`"
        )
    repo = Path(node.path).expanduser()
    if not repo.is_dir():
        raise ValueError(f"node path is not an existing directory: {repo}")
    try:
        opener = _BACKENDS[config.backend]
    except KeyError:
        avail = sorted(SUPPORTED_BACKENDS)
        raise ValueError(
            f"unknown backend {config.backend!r}; available: {avail}"
        ) from None
    try:
        return opener(node, config, runner)
    except FileNotFoundError as e:
        raise RuntimeError(
            f"backend {config.backend!r} binary not found — is it installed?"
        ) from e
    except subprocess.CalledProcessError as e:
        detail = e.stderr or e
        raise RuntimeError(
            f"backend {config.backend!r} command failed (daemon running?): {detail}"
        ) from e
