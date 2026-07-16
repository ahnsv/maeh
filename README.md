# maeh

`maeh` is a Rust CLI for the hmph/Herdr orchestration workflow. It replaces the
fragile bash helper surface with a typed command interface, structured output,
local tracker caches, compact task capsules, and a small doctor command for
operator debugging.

## Commands

```text
maeh init
maeh config path
maeh config show
maeh config emit
maeh ledger append --loop daily --event run_start --target w1 --data '{}'
maeh ledger list --loop daily
maeh state tag w1 task_url https://example/task
maeh state list
maeh board-cache put --key intake < board.json
maeh board-cache get --key intake
maeh capsule put <url> --edited <timestamp> < capsule.json
maeh capsule prompt <url>
maeh prompt kickoff --url <task-url>
maeh statusline
maeh work-hours
maeh doctor
maeh selftest
```

## Design

- deterministic local state under `MAEH_HOME` or `~/.maeh`
- line-oriented output that is easy to assert in tests and parse in logs
- compact task capsules so agents do not repeatedly pull full Notion/Linear/Jira context
- per-loop board cache TTLs matching the orchestration cadence
- explicit doctor output for path/config/backend debugging

See `docs/bash-helper-parity.md` for the mapped Bash helper surface.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo llvm-cov --all-targets --all-features --fail-under-lines 100 --fail-under-functions 100
```
