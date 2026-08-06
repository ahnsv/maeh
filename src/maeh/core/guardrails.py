from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from maeh.core.config import Config


def resolve(config: Config, home: Path) -> list[str]:
    """Effective guardrails = explicit `[review].guardrails` plus every
    `$MAEH_HOME/guardrails/*.md`, deduped by resolved path, order-stable.

    An explicit path that doesn't resolve raises (fail-closed — a configured guardrail
    must never silently drop). An empty result is allowed (operator choice). Discovered
    paths are always valid relative to the current home, so a moved `$MAEH_HOME` can't
    strand them."""
    out: list[str] = []
    seen: set[Path] = set()

    def add(p: Path) -> None:
        rp = p.resolve()
        if rp not in seen:
            seen.add(rp)
            out.append(str(rp))  # canonical absolute; matches the dedupe key

    for g in config.review.guardrails:
        gp = Path(g).expanduser()
        if not gp.exists():
            raise ValueError(
                f"configured guardrail not found: {g} — agents would run unguarded; "
                "fix [review].guardrails or run `maeh init`"
            )
        add(gp)

    d = home / "guardrails"
    if d.is_dir():
        for f in sorted(d.glob("*.md")):
            add(f)
    return out
