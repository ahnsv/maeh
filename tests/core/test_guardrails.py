import pytest

from maeh.core import guardrails
from maeh.core.config import Config, ReviewConfig


def _cfg(tmp_path, explicit=None):
    cfg = Config(maeh_home=tmp_path)
    cfg.review = ReviewConfig(guardrails=explicit or [])
    return cfg


def test_discovers_dropped_files_sorted(tmp_path):
    d = tmp_path / "guardrails"
    d.mkdir()
    (d / "b.md").write_text("b")
    (d / "a.md").write_text("a")
    assert guardrails.resolve(_cfg(tmp_path), tmp_path) == [
        str((d / "a.md").resolve()),
        str((d / "b.md").resolve()),
    ]


def test_explicit_missing_raises(tmp_path):
    with pytest.raises(ValueError):
        guardrails.resolve(_cfg(tmp_path, ["/nope/x.md"]), tmp_path)


def test_dedupes_explicit_and_discovered(tmp_path):
    d = tmp_path / "guardrails"
    d.mkdir()
    f = d / "default.md"
    f.write_text("g")
    cfg = _cfg(tmp_path, [str(f)])  # same file, explicit + discovered
    assert guardrails.resolve(cfg, tmp_path) == [str(f.resolve())]


def test_empty_allowed(tmp_path):
    assert guardrails.resolve(_cfg(tmp_path), tmp_path) == []
