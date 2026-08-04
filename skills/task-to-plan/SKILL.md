---
name: task-to-plan
description: This skill should be used when the user asks to "turn this task into a plan", "make a plan tree", "task to plan", "plan this ticket", or hands over a tracker item or raw request that needs to become executable work. Interviews and explores the repo to compile a task into a maeh plan tree.
version: 0.1.0
---

# task-to-plan

Compile a task — a tracker item (Linear/Jira/Notion) or raw natural language —
into a maeh **plan tree**: a tree whose nodes are executable actions, each with a
stable id, saved under `$MAEH_HOME`.

## When to use

Stage 1 of the maeh workflow. Use at the start of any new piece of work, before
workspaces or execution exist.

## Procedure

1. **Classify the input.** Tracker items carry their own structure — extract
   title, description, acceptance criteria. Raw natural language lacks context —
   proceed to interview.
2. **Interview to fill gaps.** Ask only what blocks planning: goal, constraints,
   success criteria, out-of-scope. Ask a few questions at a time, not a wall.
3. **Explore the repo.** Locate the files, patterns, and tests the work touches.
   Ground every planned node in a real code location.
4. **Compose the plan tree via the CLI.** `maeh plan create <plan_id> <name>`, then
   `maeh plan add <plan_id> <node_id> <name> --path <repo> --brief "<scope + acceptance>"`
   per node. Each node is the smallest independently reviewable action with a stable id.
5. **Freeze the brief.** This interview/exploration is the *only* place the fuzzy
   task detail is distilled — it is stored on the node as `--brief` and frozen.
   Execute-time capsules are a pure, deterministic render of that brief (via
   `maeh capsule …`); never re-cook the capsule at open time.

## Output

A saved plan tree (`<MAEH_HOME>/plans/<root-id>.json`) ready for
[[plan-to-workspaces]].

## Guardrails

- One node = one reviewable increment. Split anything a reviewer could reject
  independently; fold setup/scaffolding into the node whose deliverable needs it.
- Do not invent nodes with no code location — explore first, plan second.
- Stop and ask when the goal or success criteria are ambiguous.
