---
name: change-of-plan
description: This skill should be used when the user asks to "change the plan", "adjust the plan", "the review failed, fix the plan", "re-plan", or when a review verdict is failure and the plan tree needs to change or work needs to route back to Execute. Adjusts the plan tree or sends nodes back to execution.
version: 0.1.0
---

# change-of-plan

React to a review failure: adjust the plan tree, or route affected nodes back to
Execute for fixes.

## When to use

After [[review-the-increments]] returns a `fail` verdict. Stage 4 → back to
Stage 2 (Plan) or Stage 3 (Execute) of the maeh workflow.

## Procedure

1. **Read the findings.** Each is tied to a node id.
2. **Decide the routing per finding:**
   - **Fix in place** — the plan is right, the increment is wrong: mark the node
     `FAILED`, then re-run [[plan-to-workspaces]] for it.
   - **Adjust the plan** — the plan is wrong: add, remove, or re-scope nodes.
3. **Apply the change through the single writer:**
   `maeh.core.store.update_plan(home, plan_id, mutate)`, where `mutate` calls
   `maeh.core.plan.set_status` / `add_child` on the tree. Never `load`+`save`
   by hand — `update_plan` locks the plan so concurrent primary/critic can't
   clobber each other.
4. **Record why.** Note the change so [[improve-the-workflow]] can learn from it.

## Output

An updated, saved plan tree with nodes routed back to Plan or Execute.

## Guardrails

- Change the smallest thing that addresses the root cause — do not rewrite the
  whole tree for one failing node.
- Preserve stable node ids for nodes that persist; only new nodes get new ids.
- If findings reveal the goal itself was wrong, stop and escalate to the human
  rather than silently re-scoping.
