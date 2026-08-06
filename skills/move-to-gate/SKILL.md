---
name: maeh-move-to-gate
description: This skill should be used when the user asks to "move to gate", "prepare the gate summary", "make the gate one-pager", "ready for human review", or when reviewed increments need a human gate. Produces a one-page interactive gate summary for holistic human review with inline comments.
version: 0.1.0
---

# move-to-gate

Produce the one-page **gate summary** that lets a human review the work
holistically before Ship & Release.

## When to use

Stage 5 (Gate) of the maeh workflow, after [[review-the-increments]] passes.

## Procedure

1. **Locate the increments in the plan tree.** State where these specific
   increments sit in the context of the current plan tree.
2. **Write the one-pager** with four sections:
   - **Context** — what this set of increments is, and where in the plan tree.
   - **What to check** — the specific things the human should verify.
   - **Asks** — explicit questions or decisions requested from the reviewer.
   - **Changes in a nutshell** — what the agents changed, concisely.
3. **Make it interactive.** The summary must let the gate reviewer leave inline
   comments (see open question in SPEC §10 on the interactivity mechanism).
4. **Present** the one-pager to the human gate.

## Output

An interactive one-page gate summary. On approval, proceed to Ship & Release. On
comments, route back through [[change-of-plan]].

## Guardrails

- One page. If it does not fit, the increment set is too large — split it.
- Summarize; link to the full diff/artifact rather than pasting it.
- Never auto-approve — the gate is a human decision by definition.
