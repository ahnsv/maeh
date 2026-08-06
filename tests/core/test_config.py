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


def test_set_override_single_guardrail_becomes_list(tmp_path):
    cfg = load_config(tmp_path, overrides=["review.guardrails=/one/path.md"])
    assert cfg.review.guardrails == ["/one/path.md"]  # not list("/one/path.md")


def test_set_override_backend_is_validated(tmp_path):
    with pytest.raises(ValueError):
        load_config(tmp_path, overrides=["core.backend=screen"])


def test_config_to_dict_roundtrips_shape(tmp_path):
    d = config_to_dict(load_config(tmp_path))
    assert d["core"]["backend"] == "tmux"
    assert d["tui"]["status_format"]["done"] == ["✔", "green"]


def test_worktree_and_workspace_defaults(tmp_path):
    cfg = load_config(tmp_path)
    assert cfg.worktree.prefix == "maeh"
    assert cfg.worktree.location == "~/.maeh/worktrees"
    assert cfg.workspace.panes_for("tmux") == ["editor", "primary", "critic"]


def test_per_backend_panes_override(tmp_path):
    (tmp_path / "config.toml").write_text(
        '[worktree]\nlocation = ".worktrees"\n'
        '[workspace]\npanes = ["primary"]\n'
        '[workspace.herdr]\npanes = ["primary", "critic"]\n'
    )
    cfg = load_config(tmp_path)
    assert cfg.worktree.location == ".worktrees"
    assert cfg.workspace.panes_for("tmux") == ["primary"]  # default
    assert cfg.workspace.panes_for("herdr") == ["primary", "critic"]  # override


def test_unsafe_worktree_prefix_rejected(tmp_path):
    (tmp_path / "config.toml").write_text('[worktree]\nprefix = "../evil"\n')
    with pytest.raises(ValueError):
        load_config(tmp_path)


def test_default_config_template_loads(tmp_path):
    from maeh.core.config import DEFAULT_CONFIG_TOML

    (tmp_path / "config.toml").write_text(DEFAULT_CONFIG_TOML)
    cfg = load_config(tmp_path)  # the shipped default must be valid + loadable
    assert cfg.backend == "tmux"
    assert cfg.workspace.panes_for("tmux") == ["editor", "primary", "critic"]
