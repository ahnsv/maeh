# Addendum 05 — Implementation Plan (orchestrator bootstrap + guardrail discovery)

Extends `docs/PLAN.md`. CLI-tested, coverage ≥90, ruff+format clean.

---

## Task A — `core.guardrails.resolve`

**Files:** `src/maeh/core/guardrails.py`, `src/maeh/core/capsule.py`, `src/maeh/core/scaffold.py`, tests.

**Interfaces:**
- `resolve(config, home) -> list[str]`:
  - explicit = each `config.review.guardrails` path `expanduser`-d; if not `exists()` →
    `ValueError` (fail-closed).
  - discovered = `sorted((home / "guardrails").glob("*.md"))` (empty if no dir).
  - return explicit + discovered, deduped by `Path(p).resolve()`, order-stable.
- `capsule.prepare`: replace the inline missing-guardrail check + `config.review.guardrails`
  with `guardrails.resolve(config, home)`; pass the result to `capsule(...)`.
- `cli capsule` command: render with `guardrails.resolve(cfg, cfg.maeh_home)`.
- `scaffold.init_home`: drop the `guardrails = []` → absolute-path replace; write
  `DEFAULT_CONFIG_TOML` unchanged. `default.md` in the home is discovered.

**Steps:** failing tests →
1. `resolve` finds a file dropped in `<home>/guardrails/`; explicit + discovered dedupe.
2. explicit missing path → `ValueError`.
3. after `init_home`, `resolve` returns the discovered `default.md`; `config.toml` has
   `guardrails = []`.
→ implement → green → commit. (Update the Phase-A scaffold test that asserted the
pre-wired absolute path.)

---

## Task B — `core.bootstrap.orchestrator_prompt` + `maeh orchestrator`

**Files:** `src/maeh/core/bootstrap.py`, `src/maeh/cli/main.py`, tests.

**Interfaces:**
- `orchestrator_prompt(config, home) -> str`:
  - `agent = (home / "agents/orchestrator/AGENT.md")`; missing → `ValueError("run maeh init")`.
  - assemble: AGENT.md body + `\n## Active guardrails\n` + each `resolve(...)` file's
    inlined content (fenced with its filename) + `\n## Runtime\n` bullet list
    (backend, primary/critic/editor cmds, worktree prefix+location, panes).
  - deterministic (no clock/random — ast-guard like the capsule test).
- `maeh orchestrator` → `typer.echo(orchestrator_prompt(load_config(overrides), resolve_home()), nl=False)`.

**Steps:** failing tests →
1. after `init_home(tmp)`, `orchestrator_prompt` contains the AGENT.md heading, the
   default guardrail's content, and a `## Runtime` line with the backend.
2. no `AGENT.md` (uninitialized home) → `ValueError`.
3. CliRunner `maeh orchestrator` on an initialized `MAEH_HOME` exits 0 and prints the
   prompt; `no-clock/random` ast check on `bootstrap.py`.
→ implement → green → commit.

---

## Validation (live)
- `maeh init` a temp home, drop a second `guardrails/extra.md`, run `maeh capsule …` →
  both guardrails listed. `maeh orchestrator` → prints AGENT.md + inlined guardrails +
  runtime; pipe-check `maeh orchestrator | head`.

## Self-review
- `resolve` is the single guardrail source for capsule + bootstrap (DRY); fail-closed on
  explicit-missing only; empty set allowed.
- init no longer writes an absolute path → moved-home fragility gone.
- bootstrap is a pure render of files+config; harness-agnostic (stdout).
