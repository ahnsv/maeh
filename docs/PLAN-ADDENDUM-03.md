# Addendum 03 — Implementation Plan (worktree robustness)

Extends `docs/PLAN.md`. Two focused fixes in `core/workspace.py`; fake-runner tests,
coverage ≥90, ruff+format clean. Live-validate herdr `worktree open` + `--path`.

---

## Task A — C1: disambiguate the worktree location

**Files:** `src/maeh/core/workspace.py`, `tests/core/test_workspace.py`

**Interfaces:**
- `_repo_key(repo: Path) -> str` — `f"{repo.name}-{sha1(str(repo.resolve()).encode()).hexdigest()[:8]}"`. Deterministic (`hashlib`).
- `resolve_worktree(...)` central branch: `base = Path(location).expanduser() / _repo_key(repo)` (was `/ repo.name`). Project-local branch unchanged.

**Steps:** failing test — two repos with the same basename but different roots yield
different central worktree paths, and the path still ends in `<branch>`; the `~`/abs
and `.worktrees` cases still resolve. → implement → green → commit.

---

## Task B — C2: herdr open-or-create + `--path`

**Files:** `src/maeh/core/workspace.py`, `tests/core/test_workspace.py`

**Interfaces:**
- `_herdr_worktree_by_branch(run: Runner, branch: str) -> dict | None` — parse
  `herdr worktree list` (`result.worktrees[]`), return the entry whose `branch`
  matches, else `None`.
- `_open_herdr` rewritten to the 3-step flow (workspace reuse → worktree reattach →
  create). Reattach: `herdr worktree open --cwd <repo> --branch <branch> --label
  <label>`. Create now also passes `--path <resolve_worktree path>`. Both parse
  `result.workspace.workspace_id` + `result.root_pane.pane_id`; the existing seed loop
  (`_seed`, `pane split`/`pane run`) is unchanged and shared.

**Steps:** fake-runner tests —
1. **reattach:** `workspace list` empty, `worktree list` returns a `branch=maeh-n1`
   entry → assert the sequence issues `herdr worktree open … --branch maeh-n1` (not
   `create`) and seeds panes.
2. **create:** both lists empty → assert `herdr worktree create … --path <expected>`
   is issued and panes seeded.
3. **reuse:** `workspace list` has the label → returns early, no worktree/create calls
   (existing test, keep).
→ implement → green → commit.

---

## Validation (live, throwaway repo, teardown)

- Recreate the P0-#3 scenario: create a herdr worktree, close the workspace (leaving
  the checkout), then `maeh open` the same node → confirm it **reattaches** (no
  `already exists` error) and re-seeds. Confirm `--path` places the checkout under the
  configured `[worktree].location`. Tear down (`herdr worktree remove`).
- Confirm C1: two throwaway repos both named `repo` open without colliding.

---

## Self-review
- `resolve_worktree` change is covered by the collision test; hashing is deterministic
  (no random/clock — extend the capsule-style ast guard is overkill here, one asserted
  equality suffices).
- `_open_herdr` reattach vs create branch both parse the same response shape; seed loop
  untouched → capsule substitution/fail-fast behavior preserved.
- tmux path unchanged (C1 disambiguation flows through `resolve_worktree` it already uses).
