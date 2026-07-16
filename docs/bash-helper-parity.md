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
| board cache put/get/stale fallback | `maeh board-cache put|get --key <intake|revamp> [--stale]` |
| task capsule put/get/prompt | `maeh capsule put|get|prompt <url> [--edited <timestamp>]` |
| kickoff message rendering | `maeh prompt kickoff --url <task-url> [--capsule-file <path>]` |
| work-hours guard | `maeh work-hours` |
| operator diagnostics | `maeh doctor` |

## Test policy

The CLI contract is line-stable: integration tests assert stdout, stderr, and exit status for every command family and representative error path. CI fails if line or function coverage drops below 100%.
