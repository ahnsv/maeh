# maeh SPEC — Addendum 05: harness-agnostic orchestrator bootstrap + guardrail discovery

Makes "spin up any orchestrator and it picks up the maeh workflow + guardrails" true
**without a harness-specific skill/plugin**. The wiring is a `maeh` CLI command any
harness feeds to its agent; a Claude Code plugin (deferred) would just be a thin
adapter over it.

## E1 — Guardrails are discovered from `$MAEH_HOME/guardrails/`

Today `[review].guardrails` is an explicit path list — dropping a file in the home
doesn't enable it, and `maeh init` pre-wires an **absolute** path that breaks if
`$MAEH_HOME` moves.

**Fix — `core.guardrails.resolve(config, home) -> list[str]`:** the effective guardrail
set is the union of
1. explicit `[review].guardrails` (each `expanduser`-d; **missing → `ValueError`**,
   fail-closed — a configured guardrail that doesn't resolve must never silently drop),
2. every `*.md` under `$MAEH_HOME/guardrails/` (auto-discovered), sorted,

deduped by resolved path. So dropping a guardrail file "just works," and discovered
paths are always valid relative to the current home (no moved-home fragility).

- `capsule` (render + `prepare`) and the bootstrap both use `resolve` instead of
  `config.review.guardrails` directly; the fail-closed check moves into `resolve`.
- `maeh init` **stops pre-wiring** the config path — it just writes
  `guardrails/default.md` into the home; discovery picks it up. (`config.toml` keeps
  `guardrails = []`; `[review].guardrails` remains for *extra*, out-of-home guardrails.)
- Empty effective set (home dir emptied + no explicit paths) is allowed — that's an
  operator choice, not a misconfiguration.

## E2 — `maeh orchestrator` prints a self-contained bootstrap prompt

`core.bootstrap.orchestrator_prompt(config, home) -> str` assembles one portable prompt:

1. **Instructions** — the body of `$MAEH_HOME/agents/orchestrator/AGENT.md` (missing →
   `ValueError` "run `maeh init`").
2. **Active guardrails** — the **inlined content** of every `resolve(...)` guardrail
   (self-contained: the agent gets the rules, not a path it must fetch).
3. **Runtime** — live facts from config: backend, `[agents]` commands, worktree
   `prefix`/`location`, `[workspace].panes`.

Deterministic (no clock/random). `maeh orchestrator` prints it to stdout.

**Harness-agnostic usage** — any harness pipes it into its agent:
- Claude Code: a plugin subagent whose body is this output (the deferred Phase-B adapter).
- Codex / Cursor: `maeh orchestrator > AGENTS.md` (or their prompt mechanism).
- Anything: `<harness> "$(maeh orchestrator)"`.

## Interfaces

| Symbol | Where | Purpose |
|---|---|---|
| `guardrails.resolve(config, home) -> list[str]` | `core.guardrails` | explicit (fail-closed) + `$MAEH_HOME/guardrails/*.md`, deduped |
| `bootstrap.orchestrator_prompt(config, home) -> str` | `core.bootstrap` | AGENT.md + inlined guardrails + runtime |
| `maeh orchestrator` | `cli.main` | print the bootstrap prompt |

## Non-goals

- No Claude Code plugin (still a separate, deferred adapter).
- Bootstrap does not *launch* an orchestrator — it emits the prompt; the harness spawns.
- No change to how *working* agents get guardrails (capsule mechanism) beyond E1's
  discovery.
