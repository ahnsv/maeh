# Bash helper parity

`maeh` covers the deterministic helper layer and the live backend orchestration paths used by hmph/Herdr loops. Live Herdr/tmux mutation goes through backend adapters and one injectable command-runner seam, so tests can assert command shape without requiring a live terminal session.

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
| backend selection (`auto|tmux|herdr`) | typed config plus `MAEH_BACKEND`, `MAEH_HERDR_BIN`, `MAEH_TMUX_BIN`, `MAEH_TMUX_SESSION` overrides |
| backend discovery plan | `maeh backend plan` |
| normalized tmux/Herdr discovery | `maeh backend discover --fixture <adapter-output>` or explicit `--exec` |
| dry-run state reconciliation | `maeh backend reconcile --fixture <adapter-output>` or explicit `--exec` |
| backend task slot report | `maeh backend list-task-slots` emits `slot task_url status snooze_until age_secs label primary_pane critic_pane worktree` |
| backend worktree report | `maeh backend list-worktrees` |
| worktree/workspace plan | `maeh worktree plan --slot <slot> --repo <repo> --path <path> [--create] [--no-editor]` |
| worktree/workspace open | `maeh worktree open --slot <slot> --repo <repo> --path <path> [--create]` |
| spawn primary/critic agents | `maeh slot spawn --backend <backend> --slot <slot> --repo <repo> --branch <branch> --path <path> --task-url <url> --editor true|false --exec` |
| verify spawned slot | `maeh slot verify <slot>` |
| close backend slot | `maeh slot close <slot> --exec` |
| remove slot worktree | `maeh slot worktree-remove <slot> [--exec]` |
| count/filter slots | `maeh slot count --status active,blocked` or `maeh slot count --class stale` |
| snooze/block/resume slot | `maeh slot snooze <slot> --until <epoch>` / `maeh slot snooze <slot> --days <n> --status blocked` / `maeh slot block <slot> --reason <text>` / `maeh slot resume <slot>` |
| queued prompt delivery | `maeh kickoff run --slot <slot> --prompt <text>` or `maeh agent deliver <target> <prompt> --exec` |
| prompt execution verification | `maeh verify prompt --before <text> --after <text> --prompt <text>` |
| board cache put/get/stale fallback | `maeh board-cache put|get --key <intake|revamp> [--stale]` |
| task capsule put/get/prompt | `maeh capsule put|get|prompt <url> [--edited <timestamp>]` |
| kickoff message rendering | `maeh prompt kickoff --url <task-url> [--capsule-file <path>]` |
| work-hours guard | `maeh work-hours` |
| operator diagnostics | `maeh doctor` |

## Live backend boundary

The Rust surface now covers backend resolution, discovery, reconciliation, worktree/workspace open, primary/critic spawn, slot lifecycle/status/cleanup/revamp/cap wrappers, worktree removal planning/execution, prompt delivery, and prompt verification. Herdr uses `herdr worktree create|open`, `herdr agent start`, `herdr agent send`, `herdr workspace close`, and an explicit `herdr pane send-keys <pane> Enter` submit event. Tmux uses `git worktree add`, `tmux new-window`, `tmux split-window`, `tmux kill-window`, and explicit `tmux send-keys` text plus `Enter` events.

Prompt delivery policy is backend-neutral: pane text plus queued prompt becomes exactly one intent — submit queued prompt, answer a safe Codex trust/update/continue blocker, or no-op for busy/unknown panes. Adapters only translate that intent to Herdr or tmux commands.

## Test policy

The CLI contract is line-stable: integration tests assert stdout, stderr, and exit status for every command family and representative error path. Backend seam tests use a fake runner for tmux and Herdr argv/cwd/env assertions; CLI live-orchestration tests use deterministic fake `herdr`/`tmux` scripts and never require a live tmux or Herdr session. CI fails if line or function coverage drops below 100%.
