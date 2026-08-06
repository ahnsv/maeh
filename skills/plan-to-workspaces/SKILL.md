---
name: maeh-plan-to-workspaces
description: This skill should be used when the user asks to "assign workspaces", "plan to workspaces", "set up the workspaces", "start executing the plan", or is ready to turn plan-tree nodes into running development environments. Maps each executable node to a code location and opens its workspace via the configured backend.
version: 0.1.0
---

# plan-to-workspaces

Turn a saved plan tree into running work: give each executable node a code
location and open a **workspace** for it. A workspace is a git worktree hosting
one pane per role (editor + primary + critic, from `[agents]`) via the configured
backend (`tmux` or `herdr`). Drive everything through the `maeh` CLI.

## When to use

Stage 3 (Execute) of the maeh workflow, after [[task-to-plan]] has produced a
plan tree.

## Procedure (CLI-only)

1. **Inspect the plan**: `maeh -o json get <plan_id>`.
2. **Select executable nodes** — leaves (or nodes marked ready); skip containers.
3. **Give each a code location**: set `--path <repo-root>` when adding the node
   (`maeh plan add <plan_id> <node_id> <name> --path <repo>`), where `<repo>` is
   the git repo root the increment lands in.
4. **Execute each node**: `maeh open <plan_id> <node_id>`. This creates the node's
   git worktree (`<prefix>-<node.id>` per `[worktree]`), opens the backend
   workspace with a pane per role running its `[agents]` command, flips the node to
   `RUNNING`, records the handle under `$MAEH_HOME/workspaces/`, and logs the event.
   It is idempotent — re-running reuses the existing workspace, never duplicating.

## Output

One worktree-backed, multi-pane workspace per executable node, each producing an
**increment** (a PR + description + green CI, a document, or an artifact). Ready
for [[review-the-increments]].

## Guardrails

- The workspace is labelled `maeh-<node.id>` — the pairing logging and review
  depend on; never rename it.
- Do not open workspaces for container nodes or blocked nodes.
- If a backend binary is missing or its daemon is down, `maeh open` errors — stop
  and report, do not silently fall back.
- Central worktrees key on the repo basename; two repos sharing a basename can
  collide — use distinct repo roots (known v1 limitation).

## Output

One open workspace per executable node, each producing an **increment** (a PR
with description + green CI, a document, or an artifact) that corresponds to its
node. Ready for [[review-the-increments]].

## Guardrails

- The workspace handle must be the node id — never rename sessions; logging and
  review depend on that pairing.
- Do not open workspaces for container nodes or nodes still blocked.
- If a backend is unavailable, stop and report — do not silently fall back.
