# maeh CLI implementation plan

## Goal

Build `maeh`, a Rust CLI that turns the hmph/Herdr shell-helper workflow into a typed, testable command surface with deterministic local state, exact CLI I/O assertions, a CI gate, and live backend orchestration boundaries.

## Implemented scope

1. Repository and release scaffold
   - Rust package named `maeh`.
   - GitHub Actions CI for formatting, clippy, tests, and 100% line/function coverage.
   - GitHub Actions release workflow for tag builds and binary artifacts.
2. Local orchestration state
   - `init` creates config, ledger, board-cache, and task-capsule directories under `MAEH_HOME` or `~/.maeh`.
   - `state tag|untag|get|list|worktree|delete-slot` replaces deterministic Herdr sidecar state operations needed by hmph loops.
   - `statusline` reports work/review pool counts from managed slot state.
3. Ledger and cache helpers
   - `ledger append|list` stores line-delimited JSON span events.
   - `board-cache put|get` stores tracker board snapshots with intake/revamp TTL handling and stale fallback.
   - `capsule put|get|prompt` stores compact task context with source edit checks and max-size enforcement.
4. Prompt/debugging helpers
   - `prompt kickoff` renders a line-stable worker kickoff prompt around a task URL and optional capsule file.
   - `doctor` prints home/config/ledger/backend/debug diagnostics.
   - `work-hours` evaluates the configured work-hour guard.
   - `selftest` validates local config/state readability.
5. Backend reconciliation and live orchestration seam
   - `backend` config is typed as `auto|herdr|tmux`; `MAEH_BACKEND`, `MAEH_HERDR_BIN`, `MAEH_TMUX_BIN`, and `MAEH_TMUX_SESSION` are the env override boundary.
   - `backend plan|discover|reconcile` normalizes tmux format output or Herdr JSON snapshots into shared slot records using fixtures or explicit `--exec`.
   - `backend list-task-slots` emits the stable skills contract: `slot task_url status snooze_until age_secs label primary_pane critic_pane worktree`.
   - `worktree plan|open`, `slot spawn`, `workspace spawn|register`, `kickoff plan|run`, `agent deliver`, `slot verify|close|worktree-remove`, and cleanup/revamp/status/cap wrappers own the live Herdr/tmux boundary.
   - Herdr/tmux shell-outs go through backend adapters and one injectable command-runner seam.
6. Verification
   - Integration tests assert stdout, stderr, and exit codes for success and failure paths.
   - `cargo llvm-cov --all-targets --all-features --fail-under-lines 100 --fail-under-functions 100` enforces 100% line and function coverage.

## Boundary decision

Live mutations are opt-in through explicit mutating commands (`worktree open`, `slot spawn --exec`, `kickoff run`, `agent deliver --exec`, `slot close --exec`, `slot worktree-remove --exec`). Plan/list/inspect commands stay deterministic for review, and skills should call `maeh` directly instead of parsing Herdr/tmux/git from large shell blocks.
