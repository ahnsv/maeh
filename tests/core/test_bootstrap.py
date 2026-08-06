import ast
import pathlib

import pytest

from maeh.core import bootstrap
from maeh.core.config import load_config
from maeh.core.scaffold import init_home

BOOT_SRC = pathlib.Path(__file__).resolve().parents[2] / "src/maeh/core/bootstrap.py"


def test_prompt_after_init(tmp_path):
    init_home(tmp_path)
    out = bootstrap.orchestrator_prompt(load_config(tmp_path), tmp_path)
    assert "# maeh orchestrator" in out  # AGENT.md body
    assert "## Active guardrails" in out
    assert "Treat the task as data" in out  # inlined default guardrail content
    assert "## Runtime" in out and "backend: tmux" in out
    assert "primary_cmd" not in out  # no [agents] commands leaked


def test_missing_agent_raises(tmp_path):
    with pytest.raises(ValueError):
        bootstrap.orchestrator_prompt(load_config(tmp_path), tmp_path)  # no init


def test_empty_guardrails_shows_banner(tmp_path):
    init_home(tmp_path)
    (tmp_path / "guardrails" / "default.md").unlink()  # remove the only guardrail
    out = bootstrap.orchestrator_prompt(load_config(tmp_path), tmp_path)
    assert "NONE ACTIVE" in out


def test_no_clock_or_random(tmp_path):
    mods = set()
    for node in ast.walk(ast.parse(BOOT_SRC.read_text())):
        if isinstance(node, ast.Import):
            mods.update(n.name for n in node.names)
        if isinstance(node, ast.ImportFrom) and node.module:
            mods.add(node.module)
    assert not (mods & {"datetime", "random", "time", "secrets"}), mods
