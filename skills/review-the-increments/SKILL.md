---
name: review-the-increments
description: This skill should be used when the user asks to "review the increments", "review the increment", "check the work against guardrails", "review the plan output", or when workspaces have produced increments that need an agent review before the human gate. Reviews increments individually and holistically against per-repo guidelines and custom guardrails, then emits a verdict.
version: 0.1.0
---

# review-the-increments

Review the increments produced by workspaces — individually and as a whole —
against each repo's existing guidelines and maeh's custom review guardrails, then
emit a pass/fail verdict.

## When to use

Stage 4 (Review) of the maeh workflow, after [[plan-to-workspaces]] has produced
increments.

## Procedure

1. **Gather increments.** For each executed node, collect its increment (PR diff,
   document, artifact) via its `WorkspaceHandle`.
2. **Load review sources.** Per-repo guidelines (CONTRIBUTING, CLAUDE.md, lint
   config) plus maeh custom guardrails.
3. **Review each increment individually** against those sources — correctness,
   convention adherence, tests, CI status.
4. **Review holistically** — do the increments compose into the plan's intent?
   Any gaps, overlaps, or contradictions across nodes?
5. **Emit a verdict** per node and overall: `pass` or `fail` with specific,
   actionable findings tied to node ids.

## Output

A verdict. On `pass`, proceed to [[move-to-gate]]. On `fail`, hand the findings
to [[change-of-plan]].

## Guardrails

- Verify claims against the actual diff/artifact — never approve on description
  alone.
- Tie every finding to a node id so it routes correctly.
- Prefer one root-cause finding over many symptom findings.
