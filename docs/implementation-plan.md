# maeh live orchestration implementation plan

## Assumptions

- Live mutation is opt-in through mutating subcommands (`worktree open`, `spawn run`, `kickoff run`); plan subcommands stay deterministic for review.
- Herdr and tmux are both shell-command backends, but all shell-outs go through one injectable command runner.
- `MAEH_BACKEND` remains the backend override; binary paths and layout choices are config/env driven.
- No editor pane means no editor pane is planned or created; agent panes still start normally.

## Implemented plan

1. Added adapter and service boundaries for backend discovery, worktree/workspace spawn, agent kickoff, harness readiness, prompt delivery, and state persistence. Verified with unit tests over a fake command runner.
2. Added CLI commands for `backend plan|discover|reconcile`, `worktree plan|open`, `spawn plan|run`, `kickoff plan|run`, and `verify prompt`. Verified line-stable stdout/stderr without live Herdr/tmux.
3. Covered Herdr/tmux plan behavior, no-editor layout, queued prompt Enter behavior, Codex trust/update/continue unblocks, and prompt execution verification. Verified with `cargo fmt`, clippy, tests, and 100% line/function coverage.
