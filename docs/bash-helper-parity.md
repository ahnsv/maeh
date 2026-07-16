# Bash helper parity

`maeh` covers the deterministic helper layer used by the hmph/Herdr loops. It intentionally keeps live terminal/workspace mutation in the loop runner until those boundaries are migrated behind injectable command-runner tests.

| Bash helper behavior | `maeh` command |
| --- | --- |
| config path lookup | `maeh config path` |
| config initialization | `maeh init` |
| resolved config display | `maeh config show` |
| env export bridge | `maeh config emit` |
| config/state smoke check | `maeh selftest` |
| append JSONL span | `maeh ledger append --loop <name> --event <event> --target <slot> --data <json>` |
| list JSONL spans | `maeh ledger list --loop <name>` |
| Herdr sidecar tag | `maeh state tag <slot> <key> <value>` |
| Herdr sidecar untag | `maeh state untag <slot> <key>` |
| Herdr sidecar get | `maeh state get <slot> <key>` |
| managed slot listing | `maeh state list` |
| slot worktree lookup | `maeh state worktree <slot>` |
| slot removal | `maeh state delete-slot <slot>` |
| hmph statusline pools | `maeh statusline` |
| backend selection (`auto|tmux|herdr`) | typed config plus `MAEH_BACKEND`, `MAEH_HERDR_BIN`, `MAEH_TMUX_BIN` overrides |
| backend discovery plan | `maeh backend plan` |
| normalized tmux/Herdr discovery | `maeh backend discover --fixture <adapter-output>` or explicit `--exec` |
| dry-run state reconciliation | `maeh backend reconcile --fixture <adapter-output>` or explicit `--exec` |
| board cache put/get/stale fallback | `maeh board-cache put|get --key <intake|revamp> [--stale]` |
| task capsule put/get/prompt | `maeh capsule put|get|prompt <url> [--edited <timestamp>]` |
| kickoff message rendering | `maeh prompt kickoff --url <task-url> [--capsule-file <path>]` |
| work-hours guard | `maeh work-hours` |
| operator diagnostics | `maeh doctor` |

## Backend migration boundary

The safe Rust surface now covers backend resolution, read-only discovery, normalization, and dry-run reconciliation. `maeh backend plan` prints the adapter command that would be used. `maeh backend discover --fixture ...` and `maeh backend reconcile --fixture ...` are deterministic test/dev paths; `--exec` is explicit and read-only.

The live mutation paths from `scripts/orchestrator.sh` remain shell-bound for now:

- `spawn_task_window`: creating tmux windows or Herdr workspaces, splitting panes, starting agents, and writing live tags/state.
- `kickoff_agents`: delivering prompts into live agent panes.
- `verify_task_window`: capturing pane output/process state after a spawn.

Those operations should move only behind the same adapter + injected runner seam, with fake-runner tests asserting argv/cwd/env and no shared-resource teardown.

## Test policy

The CLI contract is line-stable: integration tests assert stdout, stderr, and exit status for every command family and representative error path. Backend seam tests use a fake runner for tmux and Herdr argv/cwd/env assertions; CLI backend tests use fixtures and never require live tmux or Herdr. CI fails if line or function coverage drops below 100%.
