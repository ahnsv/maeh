import json
from pathlib import Path

import pytest

from maeh.core.models import Node
from maeh.core.workspace import WorkspaceHandle, open_workspace


def test_open_workspace_tmux_creates_session_named_by_node_id():
    calls = []
    h = open_workspace(
        Node("n1", "task"), Path("/tmp"), runner=lambda c: calls.append(c) or ""
    )
    assert isinstance(h, WorkspaceHandle)
    assert h.ref == "maeh-n1"
    assert calls == [["tmux", "new-session", "-A", "-d", "-s", "maeh-n1", "-c", "/tmp"]]


def test_open_workspace_herdr_reuses_existing_by_label():
    def fake(cmd):
        assert cmd[:3] == ["herdr", "workspace", "list"], (
            "must not create when one exists"
        )
        return json.dumps(
            {"result": {"workspaces": [{"label": "maeh-n1", "workspace_id": "w9"}]}}
        )

    h = open_workspace(Node("n1", "task"), Path("/tmp"), backend="herdr", runner=fake)
    assert h == WorkspaceHandle("n1", "herdr", "w9")


def test_open_workspace_herdr_creates_when_absent():
    calls = []

    def fake(cmd):
        calls.append(cmd)
        if cmd[:3] == ["herdr", "workspace", "list"]:
            return json.dumps({"result": {"workspaces": []}})
        return json.dumps({"result": {"workspace": {"workspace_id": "wX"}}})

    h = open_workspace(Node("n1", "task"), Path("/proj"), backend="herdr", runner=fake)
    assert h == WorkspaceHandle("n1", "herdr", "wX")
    assert [
        "herdr",
        "workspace",
        "create",
        "--cwd",
        "/proj",
        "--label",
        "maeh-n1",
        "--no-focus",
    ] in calls


def test_open_workspace_unknown_backend_raises():
    with pytest.raises(ValueError):
        open_workspace(Node("n", "x"), Path("/tmp"), backend="screen")


def test_open_workspace_wraps_missing_binary():
    def fake(cmd):
        raise FileNotFoundError("tmux")

    with pytest.raises(RuntimeError, match="not found"):
        open_workspace(Node("n", "x"), Path("/tmp"), runner=fake)


def test_open_workspace_wraps_command_failure():
    import subprocess

    def fake(cmd):
        raise subprocess.CalledProcessError(1, cmd, stderr="daemon down")

    with pytest.raises(RuntimeError, match="daemon"):
        open_workspace(Node("n", "x"), Path("/tmp"), backend="herdr", runner=fake)
