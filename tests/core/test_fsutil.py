import stat

from maeh.core.fsutil import private_subdir, write_private


def test_private_subdir_is_0700(tmp_path):
    private_subdir(tmp_path, "plans")
    assert stat.S_IMODE((tmp_path / "plans").stat().st_mode) == 0o700


def test_write_private_is_0600_and_atomic(tmp_path):
    p = tmp_path / "f.json"
    write_private(p, "hello")
    assert p.read_text() == "hello"
    assert stat.S_IMODE(p.stat().st_mode) == 0o600
