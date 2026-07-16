# maeh

`maeh` is a Rust CLI for the hmph/Herdr orchestration workflow. It replaces the
fragile bash helper surface with a typed command interface, structured output,
local tracker caches, compact task capsules, and a small doctor command for
operator debugging.

## Install

Install the latest released binary:

```bash
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
```

The release workflow also publishes the installer as a release asset, so this
works after the next tagged release that includes `install.sh`:

```bash
curl -fsSL https://github.com/ahnsv/maeh/releases/latest/download/install.sh | bash
```

The repository is public, so no GitHub token is required. By default the
installer writes `maeh` to `~/.local/bin` and verifies the binary against the
release checksum before installing it. Use `--dir` to choose another location:

```bash
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash -s -- --dir /usr/local/bin
```

Pin a release with `MAEH_VERSION` or `--version`:

```bash
MAEH_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
# or
curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash -s -- --version v0.1.0
```

Verify:

```bash
maeh --help
maeh doctor
```

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

See `docs/bash-helper-parity.md` for the mapped Bash helper surface and `docs/installation.md` for install options.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo llvm-cov --all-targets --all-features --fail-under-lines 100 --fail-under-functions 100
```
