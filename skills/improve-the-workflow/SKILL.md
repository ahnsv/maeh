---
name: maeh-improve-the-workflow
description: This skill should be used when the user asks to "improve the workflow", "run a retro", "how can the next run be better", "propose a workflow improvement", or after a run completes and its telemetry should drive the next run's improvement. Reads logs, metrics, retro and improvement notes to propose one concrete, evidence-backed improvement.
version: 0.1.0
---

# improve-the-workflow

Drive maeh's self-improvement: read a run's telemetry and propose exactly one
concrete improvement for the next run, with evidence and success criteria.

## When to use

After a workflow run completes, or when reviewing how the workflow is performing.

## Sources (all read-only)

- **Logs** — `$MAEH_HOME/logs/<YYYY-MM-DD>.jsonl` (one structured record per line:
  `ts`, `level`, `event`, `plan_id`, `node_id`, `message` — order by `ts`, filter
  by `event` to find rework loops and gate rejections).
- **Metrics** — `$MAEH_HOME/metrics/<name>/<YYYY-MM-DD>.jsonl`, aggregated with
  stdlib `json` (e.g. tokens per run vs. the G1 budget).
- **Retro handoff** — the short markdown note each run emits.
- **Improvement notes** — `$MAEH_HOME/improvements/<YYYY-MM-DD>.md` (prior
  proposals).

## Procedure

1. **Revisit the previous improvement note.** Did its claim hold? Validate it
   against this run's metrics before proposing anything new.
2. **Aggregate the metrics** over the jsonl files (stdlib `json`) to find the
   biggest, evidenced problem (efficiency, rework loops, gate rejections).
3. **Propose one improvement**, stating: the goal, the problem it addresses, the
   evidence, and measurable success criteria.
4. **Write the note** to `$MAEH_HOME/improvements/<today>.md` (one paragraph).

## Output

A single dated improvement note that the next run will validate.

## Guardrails

- One improvement per run — no laundry lists.
- Every claim needs evidence from the sources above; no vibes-based proposals.
- Always validate the prior note's claim first; an unvalidated chain of
  "improvements" is just drift.
