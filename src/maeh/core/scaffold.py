from __future__ import annotations

import importlib.resources
from pathlib import Path

from maeh.core.config import DEFAULT_CONFIG_TOML
from maeh.core.fsutil import private_subdir, write_private

_DATA = importlib.resources.files("maeh").joinpath("data")


def _template(rel: str) -> str:
    return _DATA.joinpath(*rel.split("/")).read_text(encoding="utf-8")


def init_home(home: Path, *, force: bool = False) -> dict[str, str]:
    """Scaffold $MAEH_HOME with config + bundled default guardrail + orchestrator
    AGENT.md. Idempotent: an existing file is skipped unless `force`. Never touches
    plans/logs/metrics/workspaces/capsules. Returns {path: "written"|"skipped"}."""
    home = Path(home).expanduser()
    result: dict[str, str] = {}

    def place(path: Path, text: str) -> None:
        if path.exists():
            if not force:
                result[str(path)] = "skipped"
                return
            # force: back up the (possibly customized) file before overwriting.
            write_private(
                path.with_suffix(path.suffix + ".bak"), path.read_text("utf-8")
            )
            write_private(path, text)
            result[str(path)] = "written (backed up prior)"
        else:
            write_private(path, text)
            result[str(path)] = "written"

    private_subdir(home)
    guard = private_subdir(home, "guardrails") / "default.md"
    agent = private_subdir(home, "agents", "orchestrator") / "AGENT.md"

    place(guard, _template("guardrails/default.md"))
    place(agent, _template("agents/orchestrator/AGENT.md"))
    # No config pre-wire: guardrails/default.md is picked up by directory discovery
    # (guardrails.resolve), which stays valid if $MAEH_HOME moves.
    place(home / "config.toml", DEFAULT_CONFIG_TOML)
    return result
