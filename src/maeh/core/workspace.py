from __future__ import annotations

import hashlib
import json
import shlex
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from maeh.core.models import Node

if TYPE_CHECKING:
    from maeh.core.config import Config

Runner = Callable[[list[str]], str]
CapsulePaths = dict[str, str]

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


def _role_cmds(config: Config) -> list[tuple[str, str]]:
    """(role, command) pairs for the backend's panes, skipping roles with no
    [agents] command. Pane creation AND command dispatch both iterate this one
    filtered list, so pane count == command count == order."""
    out: list[tuple[str, str]] = []
    for role in config.workspace.panes_for(config.backend):
        attr = _ROLE_CMD.get(role)
        cmd = getattr(config.agents, attr, "") if attr else ""
        if cmd:
            out.append((role, cmd))
    return out


def _seed(role: str, cmd: str, capsule_paths: CapsulePaths | None) -> str:
    """Substitute `{capsule}` with the role's capsule file (shell-quoted). Refuse to
    launch a `{capsule}` command with no valid, non-empty capsule — never a literal
    `{capsule}` or a blank brief."""
    if "{capsule}" not in cmd:
        return cmd
    path = (capsule_paths or {}).get(role)
    if not path or not Path(path).is_file() or Path(path).stat().st_size == 0:
        raise ValueError(
            f"role {role!r} command uses {{capsule}} but no non-empty capsule was "
            "prepared — refusing to launch a blank agent"
        )
    return cmd.replace("{capsule}", shlex.quote(path))


def _repo_key(repo: Path) -> str:
    """Basename + short hash of the resolved repo root — unique per repo so two
    repos sharing a basename don't collide onto the same worktree dir."""
    digest = hashlib.sha1(str(repo.resolve()).encode()).hexdigest()[:8]
    return f"{repo.name}-{digest}"


def resolve_worktree(
    prefix: str, location: str, repo: Path, node_id: str
) -> tuple[Path, str]:
    """Return (worktree_path, branch). Central when location is absolute/`~`
    (`<location>/<repo>-<hash>/<branch>`), else project-local under the repo — a
    relative location is confined so it can't escape the repo. `prefix` is validated
    at config load; `node_id` is validated on the node, so `branch` is path-safe."""
    branch = f"{prefix}-{node_id}"
    if location.startswith("~") or Path(location).is_absolute():
        base = Path(location).expanduser() / _repo_key(repo)
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


def _open_tmux(
    node: Node, config: Config, run: Runner, capsules: CapsulePaths | None
) -> WorkspaceHandle:
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
        for i, (role, cmd) in enumerate(_role_cmds(config)):
            if i > 0:
                run(["tmux", "split-window", "-t", f"maeh:{window}", "-c", str(wt)])
            target = f"maeh:{window}.{i}"
            seeded = _seed(role, cmd, capsules)
            run(["tmux", "send-keys", "-t", target, "-l", "--", seeded])  # literal
            run(["tmux", "send-keys", "-t", target, "Enter"])
    return WorkspaceHandle(node.id, "tmux", window, str(wt))


def _herdr_pane_id(result: dict) -> str:
    pane = result.get("pane") or result.get("root_pane") or {}
    return pane.get("pane_id", "")


def _open_herdr(
    node: Node, config: Config, run: Runner, capsules: CapsulePaths | None
) -> WorkspaceHandle:
    label = _label(node)
    workspaces = json.loads(run(["herdr", "workspace", "list"]))["result"]["workspaces"]
    for ws in workspaces:  # 1. workspace already open → reuse (no re-split)
        if ws.get("label") == label:
            wt = (ws.get("worktree") or {}).get("checkout_path", "")
            return WorkspaceHandle(node.id, "herdr", ws["workspace_id"], wt)
    repo = Path(node.path).expanduser()
    wt_path, branch = resolve_worktree(
        config.worktree.prefix, config.worktree.location, repo, node.id
    )
    # Detect an existing checkout on disk (herdr `worktree list` doesn't track
    # custom `--path` worktrees) — keyed on the disambiguated path, not branch.
    if wt_path.exists():
        # 2. checkout exists but no open workspace → reattach (P0-#3)
        result = json.loads(
            run(["herdr", "worktree", "open", "--path", str(wt_path), "--label", label])
        )["result"]
    else:
        # 3. nothing yet → create at the disambiguated path (P0-#2)
        result = json.loads(
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
                    "--path",
                    str(wt_path),
                ]
            )
        )["result"]
    ws_id = result["workspace"]["workspace_id"]
    wt = (result["workspace"].get("worktree") or {}).get("checkout_path", "")
    pane = result["root_pane"]["pane_id"]
    for i, (role, cmd) in enumerate(_role_cmds(config)):
        if i > 0:
            # herdr: pane id is positional; direction is right|down.
            split = json.loads(
                run(["herdr", "pane", "split", pane, "--direction", "down"])
            )["result"]
            pane = _herdr_pane_id(split) or pane
        run(["herdr", "pane", "run", pane, _seed(role, cmd, capsules)])
    return WorkspaceHandle(node.id, "herdr", ws_id, wt)


_Opener = Callable[[Node, "Config", Runner, "CapsulePaths | None"], WorkspaceHandle]
_BACKENDS: dict[str, _Opener] = {"tmux": _open_tmux, "herdr": _open_herdr}

SUPPORTED_BACKENDS = frozenset(_BACKENDS)


def open_workspace(
    node: Node,
    config: Config,
    capsule_paths: CapsulePaths | None = None,
    runner: Runner = _run,
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
        return opener(node, config, runner, capsule_paths)
    except FileNotFoundError as e:
        raise RuntimeError(
            f"backend {config.backend!r} binary not found — is it installed?"
        ) from e
    except subprocess.CalledProcessError as e:
        detail = e.stderr or e
        raise RuntimeError(
            f"backend {config.backend!r} command failed (daemon running?): {detail}"
        ) from e
