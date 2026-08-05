# maeh SPEC — Addendum 03: worktree robustness (collision + reattach)

Closes the two remaining P0 correctness bugs, both hit live during e2e:
- **P0-#2** — worktree paths collide when two repos share a basename.
- **P0-#3** — a worktree/branch that exists but has no open workspace breaks
  re-`open` (`worktree create … already exists`).

## C1 — Disambiguate the worktree location by repo identity

`resolve_worktree` currently keys the central path on `repo.name` (basename):
`<location>/<repo>/<branch>` — so `~/dev/a/api` and `~/dev/b/api` collide.

**Fix:** key on repo identity, not basename. Central path becomes
`<location>/<repo.name>-<h>/<branch>` where `h = sha1(str(repo.resolve()))[:8]`
(deterministic; `hashlib`, no random). Readable prefix, unique suffix. Project-local
paths (relative `location`, under the repo itself) never collide and are unchanged.

**herdr also gets an explicit `--path`.** Today herdr manages its own worktree dir
(`~/.herdr/worktrees/<basename>/…`), which (a) collides by basename and (b) ignores
`[worktree].location`. Passing `--path <resolve_worktree path>` to `herdr worktree
create`/`open` makes herdr use maeh's disambiguated, config-driven location — closing
the collision **and** the "`[worktree].location` is a lie for herdr" gap in one change.

## C2 — herdr: open-or-create (reattach an existing worktree)

herdr `worktree create` fails if the checkout already exists but no workspace is
open (a workspace was closed, or a prior open crashed mid-sequence). The find-by-label
short-circuit misses it (label is gone with the workspace). Confirmed live: a leftover
`~/.herdr/worktrees/repo/maeh-n1` made every re-`open` fail.

**Fix — `_open_herdr` becomes open-or-create:**
1. `herdr workspace list` → a workspace labelled `maeh-<id>` → **reuse** (unchanged).
2. Else **the checkout dir already exists on disk** at the disambiguated `wt_path`
   (`wt_path.exists()`) → **reattach**: `herdr worktree open --path <wt_path>
   --label maeh-<id>` → seed panes → return.
3. Else **create**: `herdr worktree create --cwd <repo> --branch maeh-<id>
   --label maeh-<id> --path <wt_path>` → seed panes → return.

**Live-verified (herdr 0.7.5):** `worktree create --path <abs>` is honored (checkout
lands there); `worktree open --path` reattaches; both return
`result.workspace.workspace_id` + `result.root_pane.pane_id`, so seeding is identical.
Detection keys on `wt_path.exists()` (the disambiguated path), **not** `herdr worktree
list` — which was found *not* to enumerate custom-`--path` worktrees — and not on
branch name (which could match another repo). This mirrors tmux's `_ensure_worktree`
existence check.

## Migration (path scheme changed)

C1 changes the central worktree path from `<location>/<repo>/<branch>` to
`<location>/<repo>-<hash>/<branch>`. **Worktrees created under the old scheme are
orphaned** — `resolve_worktree` no longer computes their path, so nothing revisits
them, and their `maeh-<id>` branches stay checked out at the old location (which can
make a later tmux `worktree add` of the same node fail "already checked out"). maeh is
pre-release with no persisted worktree state, so no automated migration ships; operators
who ran earlier builds should `git worktree remove` / `herdr worktree remove` any stale
`<location>/<repo>/maeh-*` (and `~/.herdr/worktrees/<repo>/maeh-*`) checkouts. Automated
GC remains the tracked top ops item.

**tmux** already reuses an existing checkout (`_ensure_worktree` skips when the path
exists) and is branch-aware (`worktree add <wt> <branch>` vs `-b`), so C2 is
herdr-only; C1's disambiguation applies to both.

## Interfaces

| Symbol | Change |
|---|---|
| `resolve_worktree(prefix, location, repo, node_id)` | central base = `<loc>/<repo>-<sha1(repo.resolve())[:8]>` |
| `_repo_key(repo) -> str` | `<repo.name>-<hash>` (keyed on `repo.resolve()`; node.path should be the repo root) |
| `_open_herdr` | workspace-reuse → `wt_path.exists()` reattach (`worktree open --path`) → create (`--path`) |

## Non-goals / deferred (unchanged)

- No GC of stale worktrees/branches (still the top ops item).
- No cross-plan node-id uniqueness — disambiguation is per repo, and branch
  `maeh-<node.id>` is unique within a repo; the same node id reused across repos maps
  to different `--path` dirs, so no clash.
