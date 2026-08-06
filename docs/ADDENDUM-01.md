# maeh SPEC — Addendum 01: CLI-first workflow, worktree-backed pane workspaces

Extends `docs/SPEC.md`. Five changes, all driven by the end-to-end test run
(driving the workflow via `python -c` was the smell this fixes).

## A1 — The workflow is drivable through the `maeh` CLI alone

**Problem:** Task→Plan→Execute could only be run by importing `maeh.core` from
Python. Skills/agents (and tests) must drive it through the binary.

**New commands** (all respect global `--set` and `-o`):

| Command | Effect |
|---|---|
| `maeh plan create <id> <name>` | create + persist a root plan tree |
| `maeh plan add <id> <node_id> <name> [--parent PID] [--path DIR]` | add a node via `update_plan` |
| `maeh plan set-status <id> <node_id> <status>` | mutate a node's status via `update_plan` |
| `maeh open <id> <node_id>` | Execute: open the node's worktree-backed workspace, set it RUNNING |
| `maeh list [--filter k=v]…` | list workflows (A3) |
| `maeh default-config` | print the default config (A2) |
| `maeh config path` | print the active config path |

**Testing rule:** end-to-end / workflow tests drive the **CLI** (`typer.testing.CliRunner`),
never `maeh.core` imports. Core modules keep their unit tests (which legitimately
import core and inject fake runners for backend command construction). No test
opens a real tmux/herdr session — backend commands are asserted via an injected
runner; git-worktree logic is tested against a `git init` temp repo.

## A2 — `maeh default-config`

Prints the commented default `config.toml` (the full template, including the new
`[worktree]`/`[workspace]` sections) to **stdout**, so `maeh default-config >
~/.maeh/config.toml` scaffolds a fresh config. It never writes a file itself
(redirect-friendly, no clobber risk). The template content is the single source
that `docs/config.example.toml` is generated from.

## A3 — `maeh list [--filter k=v]…`

Lists every workflow (plan tree) under `$MAEH_HOME/plans/*.json`. Each row is a
flat attribute map:

- `id` — plan id
- `status` — the **root** node's status
- `todo` / `running` / `done` / `failed` — node counts by status

`--filter key=value` is repeatable and AND-combined; each filter matches one
attribute exactly (string compare, so `--filter status=running --filter failed=0`).
Unknown filter keys → error listing valid keys. Output honours `-o`
(plaintext table default; json/yaml for `jq`/`yq`).

## A4 — Worktree configuration

Each executable node runs in its own git worktree. New config section:

```toml
[worktree]
prefix = "maeh"                 # branch + directory prefix
location = "~/.maeh/worktrees"  # central. Use ".worktrees" for project-local.
```

`WorktreeConfig(prefix: str = "maeh", location: str = "~/.maeh/worktrees")`.

**Resolution** (`resolve_worktree(cfg, repo, node_id) -> (path, branch)`):
- `branch = f"{prefix}-{node_id}"`.
- If `location` is absolute or `~`-prefixed → **central**: `<expanduser(location)>/<repo_name>/<branch>`.
- If `location` is relative (e.g. `.worktrees`) → **project-local**: `<repo>/<location>/<branch>`.
- `repo` is the node's `path` (must be inside a git repo); its repo root is used.

## A5 — One worktree + panes per node (not N separate workspaces)

**Problem:** the test opened one flat workspace per node with no panes. A maeh
workspace is a worktree hosting a **pane per role** (editor, primary, critic),
running the role's command from `[agents]`.

New config, per-backend overridable:

```toml
[workspace]
panes = ["editor", "primary", "critic"]  # roles → panes, in order

[workspace.tmux]
panes = ["primary", "critic"]            # optional per-backend override
```

`WorkspaceConfig(panes: dict[str, list[str]])` where key `"default"` plus optional
backend keys; `panes_for(backend)` returns the override or the default. A role maps
to a command via `[agents]` (`editor`→`editor_cmd`, `primary`→`primary_cmd`,
`critic`→`critic_cmd`); a role with no command is skipped.

**herdr** (grounded in `herdr api schema`):
1. Find-or-create by label `maeh-<node.id>` (`herdr workspace list`) — idempotent.
2. Create: `herdr worktree create --branch <prefix-id> --label maeh-<id> --cwd <repo>`
   → returns `result.workspace.workspace_id` + `result.root_pane.pane_id`.
3. Run role 1 in the root pane: `herdr pane run <root_pane> <cmd>`.
4. Each further role: `herdr pane split --target <prev_pane> --direction vertical`
   → new `pane_id`; then `herdr pane run <pane_id> <cmd>`.

**tmux:**
1. Ensure worktree: `git -C <repo> worktree add <wt> -b <branch>` (skip if the
   path already exists — idempotent).
2. `tmux new-session -A -d -s maeh -c <wt>` then a window `maeh-<id>`, or reuse it;
   `tmux split-window` ×(n−1) with `-c <wt>`; `tmux send-keys <cmd> C-m` per pane.
3. Idempotent by window name `maeh-<id>`.

`WorkspaceHandle` gains `worktree: str` (the checkout path). `open_workspace`
takes the full `Config` (it needs `agents`, `worktree`, `workspace`, `backend`).

## Non-goals (unchanged from SPEC)

- No agent-output supervision yet — panes launch the role commands; maeh does not
  yet read/gate their output.
- No worktree GC/reconcile command yet (tracked with the stale-session item).
