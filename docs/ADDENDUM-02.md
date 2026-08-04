# maeh SPEC — Addendum 02: Task capsules (deterministic, CLI-rendered)

Extends `docs/SPEC.md` and closes P0-#1 (Execute provisions panes but never hands
the agents the work). Fuzzy interpretation happens **once at plan time and is
stored as structured data**; capsule **assembly is a pure render** — no LLM, no
clock, no randomness — so the same node always yields the same capsule.

## B1 — Node carries a plan-time `brief`

`Node` gains `brief: str | None` — the node's task detail (scope, acceptance
criteria) captured when the plan is built. Persisted in the plan JSON, round-tripped.

`maeh plan add <plan_id> <node_id> <name> [--brief TEXT] [--path DIR] [--parent PID]`.

The `task-to-plan` skill is where the fuzzy extraction lives (reading the ticket,
distilling scope); its output is written to `--brief`, then frozen. The LLM never
re-touches it.

## B2 — `core.capsule(...)` is a pure template

`capsule(tree: PlanTree, node_id: str, role: str, guardrails: list[str]) -> str`
renders deterministic Markdown from data already in the tree + config:

- **Goal** — the plan root's name.
- **Context** — the ancestor chain root→node (where this node sits).
- **Task** — the node's `name` + `brief`.
- **Guardrails** — the `[review].guardrails` paths + a "follow this repo's
  conventions" line.
- **Role framing** — first-class for `primary` ("implement, produce the increment"),
  `critic` ("review the primary's work against task + guardrails"), and `editor`
  ("editor pane, brief for reference"); any *other* role → a neutral brief.

Constraints: no `datetime`/`random`; byte-identical for identical inputs (golden
test). `role` is `require_safe_segment`-validated in `write_capsule` (it lands in a
filename); an unrecognized-but-safe role still renders (neutral brief, never an error).

## B3 — `maeh capsule` CLI command

`maeh capsule <plan_id> <node_id> [--role primary]` prints the rendered capsule to
stdout (respects `-o`: `plaintext` = the Markdown, `json` = `{role, node_id, text}`
for piping). Read-only, deterministic, zero tokens.

## B4 — `maeh open` seeds each pane with its role capsule

Per executable node, `maeh open` writes each role's capsule to a private file
under `$MAEH_HOME` (not the repo — no `git status` pollution):

`$MAEH_HOME/capsules/<plan_id>/<node_id>-<role>.md`  (0600, ids validated).

The role's `[agents]` command may contain a `{capsule}` placeholder, which maeh
substitutes with that file's absolute path before running it in the pane — e.g.
`primary_cmd = "pi {capsule}"` → `pi /…/capsules/<plan>/<node>-primary.md`. A
command with no `{capsule}` runs unchanged (the file is still written, so the
operator/agent can read it). This keeps injection **deterministic and race-free**
— no timing-dependent `send-text` into a maybe-ready agent.

## Safety notes

- A `{capsule}` command with no non-empty capsule prepared for its role **fails the
  open** (never launches `pi {capsule}` literally or an agent with a blank brief).
- The substituted path is `shlex.quote`-d before it enters the pane command.
- `brief`/`name` are untrusted, so the capsule fences them as an indented (inert)
  block with a preamble that the Role/Guardrails sections are the only authoritative
  instructions — injected `##`/fences can't forge sibling sections.

## Non-goals

- maeh does not parse or supervise agent output (unchanged).
- **Stale-on-reuse:** a capsule is rendered at `open` from the frozen `brief`. An
  already-open workspace is reused idempotently and its pane is **not** re-seeded, so
  a changed `brief` only reaches a running node after teardown + re-`open` (see the
  `change-of-plan` skill). Auto-reseed/GC is deferred.

## Interfaces summary

| Symbol | Where | Purpose |
|---|---|---|
| `Node.brief: str \| None` | `core.models` | frozen plan-time task detail |
| `capsule(tree, node_id, role, guardrails) -> str` | `core.capsule` | pure render |
| `write_capsule(home, plan_id, node_id, role, text) -> Path` | `core.store` | 0600 file under `$MAEH_HOME/capsules/` |
| `maeh capsule <plan> <node> [--role]` | `cli.main` | inspect the capsule |
| `maeh plan add … --brief` | `cli.main` | capture the brief |
| `{capsule}` substitution in `_role_cmds`/open | `core.workspace` | seed the pane deterministically |
