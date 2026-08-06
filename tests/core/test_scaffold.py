import stat

from maeh.core import guardrails
from maeh.core.config import load_config
from maeh.core.scaffold import init_home


def test_init_creates_files_and_guardrail_is_discovered(tmp_path):
    res = init_home(tmp_path)
    guard = tmp_path / "guardrails" / "default.md"
    agent = tmp_path / "agents" / "orchestrator" / "AGENT.md"
    cfg = tmp_path / "config.toml"
    assert guard.exists() and agent.exists() and cfg.exists()
    assert stat.S_IMODE(guard.stat().st_mode) == 0o600
    assert stat.S_IMODE((tmp_path / "guardrails").stat().st_mode) == 0o700
    assert res[str(cfg)] == "written"
    assert "guardrail" in guard.read_text().lower()
    # config is NOT pre-wired (guardrails = []); the file is picked up by discovery.
    loaded = load_config(tmp_path)
    assert loaded.backend == "tmux"
    assert loaded.review.guardrails == []
    assert guardrails.resolve(loaded, tmp_path) == [str(guard.resolve())]


def test_init_idempotent_then_force_backs_up(tmp_path):
    init_home(tmp_path)
    res2 = init_home(tmp_path)
    assert set(res2.values()) == {"skipped"}
    guard = tmp_path / "guardrails" / "default.md"
    guard.write_text("MINE")  # user customization
    res3 = init_home(tmp_path, force=True)
    assert "backed up" in res3[str(guard)]
    assert (tmp_path / "guardrails" / "default.md.bak").read_text() == "MINE"
    assert guard.read_text() != "MINE"  # refreshed from bundled template


def test_init_does_not_touch_state_dirs(tmp_path):
    init_home(tmp_path)
    assert not (tmp_path / "plans").exists()
    assert not (tmp_path / "logs").exists()
    assert not (tmp_path / "capsules").exists()
