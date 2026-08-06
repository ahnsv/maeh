import json

import pytest

from maeh.core.telemetry import emit_metric, log

TS = "2026-08-03T09:15:00Z"


def test_log_writes_structured_record(tmp_path):
    log(tmp_path, "started", plan_id="p1", node_id="n1", ts=TS, event="execute")
    rec = json.loads((tmp_path / "logs" / "2026-08-03.jsonl").read_text().strip())
    assert rec == {
        "ts": TS,
        "level": "info",
        "event": "execute",
        "plan_id": "p1",
        "node_id": "n1",
        "message": "started",
    }


def test_log_record_survives_newlines_in_message(tmp_path):
    log(tmp_path, "a\nfake forged", plan_id="p1", node_id="n1", ts=TS)
    lines = (tmp_path / "logs" / "2026-08-03.jsonl").read_text().splitlines()
    assert len(lines) == 1 and json.loads(lines[0])["message"] == "a\nfake forged"


def test_emit_metric_appends_jsonl(tmp_path):
    emit_metric(tmp_path, "tokens", {"n": 42}, ts=TS)
    emit_metric(tmp_path, "tokens", {"n": 7}, ts=TS)
    lines = (
        (tmp_path / "metrics" / "tokens" / "2026-08-03.jsonl").read_text().splitlines()
    )
    assert [json.loads(line)["n"] for line in lines] == [42, 7]


def test_emit_metric_rejects_unsafe_name(tmp_path):
    with pytest.raises(ValueError):
        emit_metric(tmp_path, "../evil", {"n": 1}, ts=TS)
