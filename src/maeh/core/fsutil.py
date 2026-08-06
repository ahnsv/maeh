from __future__ import annotations

import os
from pathlib import Path


def private_subdir(home: Path, *parts: str) -> Path:
    d = home
    d.mkdir(parents=True, exist_ok=True)
    d.chmod(0o700)
    for part in parts:
        d = d / part
        d.mkdir(exist_ok=True)
        d.chmod(0o700)
    return d


def write_private(path: Path, text: str) -> None:
    tmp = path.with_name(path.name + ".tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(text)
    os.replace(tmp, path)  # atomic; old file intact on crash


def append_private(path: Path, line: str) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(line)
