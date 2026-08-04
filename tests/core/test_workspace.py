import json
import subprocess
from pathlib import Path

import pytest

from maeh.core.config import Config
from maeh.core.models import Node
from maeh.core.workspace import WorkspaceHandle, open_workspace, resolve_worktree


def _repo(tmp_path: Path) -> Path:
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    return tmp_path


def _cfg(tmp_path: Path, backend: str) -> Config:
    cfg = Config(maeh_home=tmp_path / "home")
    cfg.backend = backend
    cfg.worktree.location = "~/.maeh/worktrees"  # central, avoids polluting the repo
    return cfg


def test_resolve_worktree_central_vs_local(tmp_path):
    repo = tmp_path / "myrepo"
    central, branch = resolve_worktree("maeh", "~/wt", repo, "n1")
    assert branch == "maeh-n1"
    assert central == Path.home() / "wt" / "myrepo" / "maeh-n1"
    local, _ = resolve_worktree("maeh", ".worktrees", repo, "n1")
    assert local == (repo / ".worktrees" / "maeh-n1")


def test_resolve_worktree_rejects_escape(tmp_path):
    with pytest.raises(ValueError):
        resolve_worktree("maeh", "../../etc", tmp_path / "repo", "n1")


def test_open_requires_existing_path(tmp_path):
    with pytest.raises(ValueError):
        open_workspace(Node("n1", "x"), _cfg(tmp_path, "tmux"))  # node.path is None


def test_capsule_substituted_into_pane_command(tmp_path):
    repo = _repo(tmp_path)
    cap = tmp_path / "cap.md"
    cap.write_text("hi")
    calls: list[list[str]] = []
    cfg = _cfg(tmp_path, "tmux")
    cfg.agents.primary_cmd = "pi {capsule}"
    cfg.workspace.panes["default"] = ["primary"]

    def fake(cmd):
        calls.append(cmd)
        return "" if cmd[:2] != ["tmux", "list-windows"] else ""

    open_workspace(
        Node("n1", "t", path=str(repo)), cfg, {"primary": str(cap)}, runner=fake
    )
    sent = [c for c in calls if c[:2] == ["tmux", "send-keys"] and "-l" in c]
    assert sent and sent[0][-1] == f"pi {cap}"  # {capsule} -> quoted path


def test_open_refuses_blank_capsule(tmp_path):
    repo = _repo(tmp_path)
    cfg = _cfg(tmp_path, "tmux")
    cfg.agents.primary_cmd = "pi {capsule}"
    cfg.workspace.panes["default"] = ["primary"]
    with pytest.raises((ValueError, RuntimeError)):  # {capsule} but no path prepared
        open_workspace(Node("n1", "t", path=str(repo)), cfg, {}, runner=lambda c: "")


def test_tmux_creates_worktree_window_and_panes(tmp_path):
    repo = _repo(tmp_path)
    calls: list[list[str]] = []

    def fake(cmd):
        calls.append(cmd)
        if cmd[:2] == ["tmux", "list-windows"]:
            return ""  # no windows yet
        if cmd[3:5] == ["branch", "--list"]:
            return ""  # branch absent
        return ""

    cfg = _cfg(tmp_path, "tmux")
    h = open_workspace(Node("n1", "task", path=str(repo)), cfg, runner=fake)
    assert h.backend == "tmux" and h.ref == "maeh-n1"
    assert any(c[:3] == ["git", "-C", str(repo)] and "worktree" in c for c in calls)
    assert [
        "tmux",
        "new-window",
        "-t",
        "maeh",
        "-n",
        "maeh-n1",
        "-c",
        h.worktree,
    ] in calls
    # three roles -> two splits + three literal send-keys
    assert sum(1 for c in calls if c[:2] == ["tmux", "split-window"]) == 2
    assert sum(1 for c in calls if c[:2] == ["tmux", "send-keys"] and "-l" in c) == 3


def test_tmux_idempotent_when_window_exists(tmp_path):
    repo = _repo(tmp_path)
    calls: list[list[str]] = []

    def fake(cmd):
        calls.append(cmd)
        if cmd[:2] == ["tmux", "list-windows"]:
            return "maeh-n1\n"  # already there
        if cmd[3:5] == ["branch", "--list"]:
            return "  maeh-n1\n"
        return ""

    h = open_workspace(
        Node("n1", "task", path=str(repo)), _cfg(tmp_path, "tmux"), runner=fake
    )
    assert h.ref == "maeh-n1"
    assert not any(
        c[:2] == ["tmux", "new-window"] for c in calls
    )  # no duplicate window
    assert not any(c[:2] == ["tmux", "split-window"] for c in calls)


def test_herdr_reuses_existing_by_label(tmp_path):
    def fake(cmd):
        assert cmd[:3] == ["herdr", "workspace", "list"]
        return json.dumps(
            {
                "result": {
                    "workspaces": [
                        {
                            "label": "maeh-n1",
                            "workspace_id": "w9",
                            "worktree": {"checkout_path": "/wt/x"},
                        },
                    ]
                }
            }
        )

    cfg = _cfg(tmp_path, "herdr")
    h = open_workspace(Node("n1", "x", path=str(_repo(tmp_path))), cfg, runner=fake)
    assert h == WorkspaceHandle("n1", "herdr", "w9", "/wt/x")


def test_herdr_creates_worktree_and_panes(tmp_path):
    calls: list[list[str]] = []

    def fake(cmd):
        calls.append(cmd)
        if cmd[:3] == ["herdr", "workspace", "list"]:
            return json.dumps({"result": {"workspaces": []}})
        if cmd[:3] == ["herdr", "worktree", "create"]:
            return json.dumps(
                {
                    "result": {
                        "workspace": {
                            "workspace_id": "wX",
                            "worktree": {"checkout_path": "/wt/wX"},
                        },
                        "root_pane": {"pane_id": "wX:p1"},
                    }
                }
            )
        if cmd[:3] == ["herdr", "pane", "split"]:
            return json.dumps({"result": {"pane": {"pane_id": "wX:pN"}}})
        return ""

    cfg = _cfg(tmp_path, "herdr")
    h = open_workspace(Node("n1", "x", path=str(_repo(tmp_path))), cfg, runner=fake)
    assert h == WorkspaceHandle("n1", "herdr", "wX", "/wt/wX")
    assert [
        "herdr",
        "worktree",
        "create",
        "--cwd",
        str(tmp_path),
        "--branch",
        "maeh-n1",
        "--label",
        "maeh-n1",
    ] in calls
    assert sum(1 for c in calls if c[:3] == ["herdr", "pane", "split"]) == 2
    assert sum(1 for c in calls if c[:3] == ["herdr", "pane", "run"]) == 3


def test_unknown_backend_raises(tmp_path):
    cfg = _cfg(tmp_path, "tmux")
    cfg.backend = "screen"
    with pytest.raises(ValueError):
        open_workspace(Node("n", "x", path=str(_repo(tmp_path))), cfg)


def test_wraps_command_failure(tmp_path):
    def fake(cmd):
        raise subprocess.CalledProcessError(1, cmd, stderr="boom")

    with pytest.raises(RuntimeError, match="failed"):
        open_workspace(
            Node("n", "x", path=str(_repo(tmp_path))),
            _cfg(tmp_path, "herdr"),
            runner=fake,
        )
