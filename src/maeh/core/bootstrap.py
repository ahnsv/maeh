from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from maeh.core import guardrails

if TYPE_CHECKING:
    from maeh.core.config import Config


def orchestrator_prompt(config: Config, home: Path) -> str:
    """Self-contained, harness-agnostic orchestrator bootstrap prompt: the AGENT.md
    body + inlined guardrail contents + runtime facts. Deterministic (no clock/random).

    Note: does NOT print `[agents]` commands — those may embed secrets and the
    orchestrator drives `maeh open` rather than launching agents itself."""
    agent = home / "agents" / "orchestrator" / "AGENT.md"
    if not agent.is_file():
        raise ValueError(
            f"orchestrator AGENT.md not found at {agent} — run `maeh init`"
        )

    parts = [agent.read_text("utf-8").rstrip(), "", "## Active guardrails", ""]
    paths = guardrails.resolve(config, home)
    if paths:
        for p in paths:
            parts.append(f"### {Path(p).name}")
            parts.append("")
            parts.append(Path(p).read_text("utf-8").rstrip())
            parts.append("")
    else:
        parts.append(
            "**NONE ACTIVE — working agents run UNGUARDED.** Add a guardrail to"
        )
        parts.append(
            "`$MAEH_HOME/guardrails/` or `[review].guardrails` before executing."
        )
        parts.append("")

    parts += [
        "## Runtime",
        "",
        f"- backend: {config.backend}",
        f"- worktree: {config.worktree.prefix} @ {config.worktree.location}",
        f"- panes (roles): {config.workspace.panes_for(config.backend)}",
        "",
    ]
    return "\n".join(parts)
