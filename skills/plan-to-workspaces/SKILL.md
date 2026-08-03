---
name: plan-to-workspaces
description: This skill should be used when the user asks to "assign workspaces", "plan to workspaces", "set up the workspaces", "start executing the plan", or is ready to turn plan-tree nodes into running development environments. Maps each executable node to a code location and opens its workspace via the configured backend.
version: 0.1.0
---

# plan-to-workspaces

Turn a saved plan tree into running work: assign each executable node to a code
location and open a **workspace** (editor + primary + critic) for it via the
configured backend (`tmux` or `herdr`).

## When to use

Stage 3 (Execute) of the maeh workflow, after [[task-to-plan]] has produced a
plan tree.

## Procedure

1. **Load the plan tree** with `maeh.core.store.load_plan(plan_id, home)` and the
   config with `maeh.core.config.load_config()`.
2. **Select executable nodes.** Walk the tree; a leaf (or a node marked ready) is
   executable. Skip container nodes.
3. **Resolve a code location** per node — the directory/repo the increment lands
   in — and set it on `node.path` (via `maeh.core.store.update_plan`). Ground it
   in the exploration from [[task-to-plan]].
4. **Open the workspace** with
   `maeh.core.workspace.open_workspace(node, Path(node.path))`. The returned
   `WorkspaceHandle.ref` is the session id (e.g. `maeh-<node.id>`).
5. **Set status.** Mark the node `RUNNING` through
   `maeh.core.store.update_plan(home, plan_id, lambda t: set_status(t, node.id, RUNNING))`
   — the locked single writer, never a bare `load`+`save`.

## Output

One open workspace per executable node, each producing an **increment** (a PR
with description + green CI, a document, or an artifact) that corresponds to its
node. Ready for [[review-the-increments]].

## Guardrails

- The workspace handle must be the node id — never rename sessions; logging and
  review depend on that pairing.
- Do not open workspaces for container nodes or nodes still blocked.
- If a backend is unavailable, stop and report — do not silently fall back.
