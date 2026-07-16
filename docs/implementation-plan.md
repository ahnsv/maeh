# maeh initial CLI implementation

## Goal

Build `maeh`, a Rust CLI that turns the current hmph/Herdr shell-helper workflow into a typed, testable command surface with deterministic local state, exact CLI I/O assertions, a CI gate, and a tag-driven release pipeline.

## Implemented scope

1. Repository and release scaffold
   - Rust package named `maeh`.
   - GitHub Actions CI for formatting, clippy, tests, and 100% line coverage.
   - GitHub Actions release workflow for tag builds and binary artifacts.
2. Local orchestration state
   - `init` creates config, ledger, board-cache, and task-capsule directories under `MAEH_HOME` or `~/.maeh`.
   - `state tag|untag|get|list|worktree|delete-slot` replaces the deterministic Herdr sidecar state operations needed by the hmph loops.
   - `statusline` reports the work/review pool counts from managed slot state.
3. Ledger and cache helpers
   - `ledger append|list` stores line-delimited JSON span events.
   - `board-cache put|get` stores tracker board snapshots with intake/revamp TTL handling and stale fallback.
   - `capsule put|get|prompt` stores compact task context with source edit checks and max-size enforcement.
4. Prompt/debugging helpers
   - `prompt kickoff` renders a line-stable worker kickoff prompt around a task URL and optional capsule file.
   - `doctor` prints home/config/ledger/backend/debug diagnostics.
   - `work-hours` evaluates the configured work-hour guard.
   - `selftest` validates local config/state readability.
5. Backend reconciliation seam
   - `backend` config is typed as `auto|herdr|tmux`; `MAEH_BACKEND`, `MAEH_HERDR_BIN`, and `MAEH_TMUX_BIN` are the env override boundary.
   - `backend plan` prints the read-only discovery command selected for the resolved backend.
   - `backend discover` normalizes tmux format output or Herdr JSON snapshots into shared slot records using fixtures or explicit `--exec`.
   - `backend reconcile` compares normalized backend records with local state and prints dry-run operation plans instead of mutating slots.
6. Verification
   - Integration tests assert stdout, stderr, and exit codes for success and failure paths.
   - `cargo llvm-cov --all-targets --all-features --fail-under-lines 100 --fail-under-functions 100` enforces 100% line and function coverage.

## Boundary decision

The first reconciliation slice keeps live tmux/Herdr mutation outside the default path. The CLI owns the file-backed state, cache, ledger, prompt, diagnostics, typed backend resolution, read-only discovery, and dry-run reconciliation layer that was most fragile in bash. Live workspace creation, prompt delivery into panes, and post-spawn verification remain in the orchestrator loop until those boundaries can be migrated behind the adapter + injected command-runner tests without shared-resource teardown risk.
