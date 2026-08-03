from typer.testing import CliRunner

from maeh.cli.main import app
from maeh.core.models import Node, PlanTree
from maeh.core.store import save_plan

runner = CliRunner()


def test_config_command_json(tmp_path, monkeypatch):
    monkeypatch.setenv("MAEH_HOME", str(tmp_path))
    result = runner.invoke(app, ["-o", "json", "config"])
    assert result.exit_code == 0
    assert '"backend": "tmux"' in result.stdout


def test_config_command_set_override_plaintext(tmp_path, monkeypatch):
    monkeypatch.setenv("MAEH_HOME", str(tmp_path))
    result = runner.invoke(app, ["--set", "agents.primary_cmd=codex", "config"])
    assert result.exit_code == 0
    assert "agents.primary_cmd = codex" in result.stdout


def test_get_command_yaml(tmp_path, monkeypatch):
    monkeypatch.setenv("MAEH_HOME", str(tmp_path))
    save_plan(PlanTree(Node("p1", "root")), tmp_path)
    result = runner.invoke(app, ["-o", "yaml", "get", "p1"])
    assert result.exit_code == 0
    assert "id: p1" in result.stdout
