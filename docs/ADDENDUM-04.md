# maeh SPEC — Addendum 04: `maeh init` + bundled runtime data

Makes maeh's runtime data (config, default guardrail, orchestrator instructions)
installable, so any agent/harness driving maeh picks up guardrails deterministically
after `pipx install maeh`. Harness-agnostic (the runtime half of the deployment plan;
a Claude Code plugin is a separate delivery layer and out of scope here).

## D1 — Bundle runtime data as package data

Move the canonical templates under the package so they ship in the wheel and are
readable via `importlib.resources`:

- `src/maeh/data/guardrails/default.md` — the default working-agent guardrail.
- `src/maeh/data/agents/orchestrator/AGENT.md` — orchestrator instructions.

The top-level `guardrails/` and `agents/` copies are removed (single source of truth).
`pyproject` must include non-`.py` package data in the wheel (hatchling: ensure
`src/maeh/data/**` ships).

## D2 — `core.scaffold.init_home(home, *, force=False) -> dict[str,str]`

Scaffolds `$MAEH_HOME` (harness-agnostic core logic; CLI is a thin wrapper):

1. Create `$MAEH_HOME`, `guardrails/`, `agents/orchestrator/` as `0700` (via `fsutil`).
2. Write `guardrails/default.md` and `agents/orchestrator/AGENT.md` from package data
   (`0600`).
3. Write `config.toml` from `DEFAULT_CONFIG_TOML`, with `[review].guardrails`
   **pre-wired** to the absolute path of the scaffolded `guardrails/default.md` (so the
   capsule points working agents at a path that actually resolves).

Idempotent: an existing file is **skipped** (never clobbered) unless `force=True`.
Returns a map `{path: "written"|"skipped"}`. Never destroys plans/logs/metrics.

## D3 — `maeh init [--force]`

Thin CLI wrapper: resolves home (`--home`/`$MAEH_HOME`/`~/.maeh`), calls `init_home`,
prints each `written`/`skipped` path. After `pipx install maeh`, `maeh init` is the one
step that makes guardrails + orchestrator instructions live under `$MAEH_HOME`.

## Interfaces

| Symbol | Where | Purpose |
|---|---|---|
| `src/maeh/data/{guardrails/default.md, agents/orchestrator/AGENT.md}` | package data | bundled templates |
| `scaffold.init_home(home, *, force=False) -> dict[str,str]` | `core.scaffold` | idempotent scaffold; guardrail pre-wired into config |
| `maeh init [--force]` | `cli.main` | run the scaffold |

## Non-goals

- No Claude Code plugin / marketplace here (separate delivery layer).
- `init` does not touch `plans/`, `logs/`, `metrics/`, `workspaces/`, `capsules/`.
- No auto-run of `init` on other commands — explicit, so it never surprises an
  existing `$MAEH_HOME`.
