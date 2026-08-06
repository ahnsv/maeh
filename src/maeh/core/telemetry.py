from __future__ import annotations

import json
from pathlib import Path

from maeh.core.fsutil import append_private, private_subdir
from maeh.core.models import require_safe_segment


def log(
    home: Path,
    message: str,
    *,
    plan_id: str,
    node_id: str,
    ts: str,
    level: str = "info",
    event: str = "log",
) -> None:
    d = private_subdir(home, "logs")
    rec = {
        "ts": ts,
        "level": level,
        "event": event,
        "plan_id": plan_id,
        "node_id": node_id,
        "message": message,
    }
    append_private(d / f"{ts[:10]}.jsonl", json.dumps(rec, ensure_ascii=False) + "\n")


def emit_metric(home: Path, name: str, value: dict, *, ts: str) -> None:
    require_safe_segment(name)  # name is interpolated into the path
    d = private_subdir(home, "metrics", name)
    append_private(
        d / f"{ts[:10]}.jsonl",
        json.dumps({"ts": ts, **value}, ensure_ascii=False) + "\n",
    )
