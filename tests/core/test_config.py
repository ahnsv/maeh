import pytest

from maeh.core.config import config_to_dict, load_config


def test_defaults_when_no_file(tmp_path):
    cfg = load_config(tmp_path)
    assert cfg.backend == "tmux"
    assert cfg.agents.primary_cmd == "claude"
    assert cfg.tui.status_format["done"] == ("✔", "green")
    assert cfg.limits.max_concurrent_workspaces == 3


def test_file_overrides(tmp_path):
    (tmp_path / "config.toml").write_text(
        '[agents]\nprimary_cmd = "codex"\n'
        '[tui.status_format]\ndone = ["✓", "cyan"]\n'
        '[review]\nguardrails = ["g.md"]\n'
    )
    cfg = load_config(tmp_path)
    assert cfg.agents.primary_cmd == "codex"
    assert cfg.agents.critic_cmd == "claude"  # untouched default kept
    assert cfg.tui.status_format["done"] == ("✓", "cyan")
    assert cfg.tui.status_format["failed"] == ("✗", "red")  # untouched default kept
    assert cfg.review.guardrails == ["g.md"]


def test_herdr_backend_is_supported(tmp_path):
    (tmp_path / "config.toml").write_text('[core]\nbackend = "herdr"\n')
    assert load_config(tmp_path).backend == "herdr"


def test_unsupported_backend_rejected(tmp_path):
    (tmp_path / "config.toml").write_text('[core]\nbackend = "screen"\n')
    with pytest.raises(ValueError):
        load_config(tmp_path)


def test_set_overrides_apply_and_coerce(tmp_path):
    cfg = load_config(
        tmp_path,
        overrides=["agents.primary_cmd=codex", "limits.max_concurrent_workspaces=5"],
    )
    assert cfg.agents.primary_cmd == "codex"  # no file needed
    assert cfg.limits.max_concurrent_workspaces == 5  # coerced to int


def test_set_override_backend_is_validated(tmp_path):
    with pytest.raises(ValueError):
        load_config(tmp_path, overrides=["core.backend=screen"])


def test_config_to_dict_roundtrips_shape(tmp_path):
    d = config_to_dict(load_config(tmp_path))
    assert d["core"]["backend"] == "tmux"
    assert d["tui"]["status_format"]["done"] == ["✔", "green"]
