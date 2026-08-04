"""CLI-only tests — the workflow is driven through `maeh` (CliRunner), never by
importing maeh.core. Backend side effects are stubbed at the CLI boundary."""

import tomllib

from typer.testing import CliRunner

from maeh.cli.main import app

runner = CliRunner()


def _run(env_home, *args):
    return runner.invoke(app, list(args), env={"MAEH_HOME": str(env_home)})


def test_default_config_prints_loadable_toml(tmp_path):
    r = _run(tmp_path, "default-config")
    assert r.exit_code == 0
    data = tomllib.loads(r.stdout)  # must be valid TOML
    assert data["core"]["backend"] == "tmux"
    assert data["workspace"]["panes"] == ["editor", "primary", "critic"]


def test_config_view_json(tmp_path):
    r = _run(tmp_path, "-o", "json", "config")
    assert r.exit_code == 0 and '"backend": "tmux"' in r.stdout


def test_config_set_override(tmp_path):
    r = _run(tmp_path, "--set", "agents.primary_cmd=codex", "config")
    assert r.exit_code == 0 and "agents.primary_cmd = codex" in r.stdout


def test_workflow_plan_lifecycle_cli_only(tmp_path):
    assert _run(tmp_path, "plan", "create", "wf", "Build it").exit_code == 0
    assert _run(tmp_path, "plan", "add", "wf", "n1", "First").exit_code == 0
    assert _run(tmp_path, "plan", "add", "wf", "n2", "Second").exit_code == 0
    assert _run(tmp_path, "plan", "set-status", "wf", "n1", "done").exit_code == 0

    got = _run(tmp_path, "-o", "json", "get", "wf")
    assert got.exit_code == 0
    assert '"id": "n1"' in got.stdout and '"status": "done"' in got.stdout

    listed = _run(tmp_path, "-o", "json", "list")
    assert listed.exit_code == 0
    assert '"id": "wf"' in listed.stdout and '"done": 1' in listed.stdout


def test_list_filter(tmp_path):
    _run(tmp_path, "plan", "create", "wf", "x")
    assert (
        '"id": "wf"'
        in _run(tmp_path, "-o", "json", "list", "--filter", "status=todo").stdout
    )
    assert _run(
        tmp_path, "-o", "json", "list", "--filter", "status=done"
    ).stdout.strip() in ("[]",)


def test_list_unknown_filter_key_errors(tmp_path):
    r = _run(tmp_path, "list", "--filter", "bogus=1")
    assert r.exit_code != 0


def test_open_command_stubbed(tmp_path, monkeypatch):
    from maeh.core.workspace import WorkspaceHandle

    monkeypatch.setattr(
        "maeh.cli.main.open_workspace",
        lambda node, cfg: WorkspaceHandle(node.id, "tmux", "maeh-n1", "/wt/maeh-n1"),
    )
    _run(tmp_path, "plan", "create", "wf", "x")
    _run(tmp_path, "plan", "add", "wf", "n1", "First", "--path", str(tmp_path))
    r = _run(tmp_path, "-o", "json", "open", "wf", "n1")
    assert r.exit_code == 0 and '"ref": "maeh-n1"' in r.stdout
    # node flipped to RUNNING and the handle recorded
    assert '"status": "running"' in _run(tmp_path, "-o", "json", "get", "wf").stdout
