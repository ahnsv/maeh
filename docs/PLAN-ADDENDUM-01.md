# Addendum 01 — Implementation Plan

Extends `docs/PLAN.md`. Same conventions: core holds logic, CLI is presentation,
tests drive the CLI for workflow paths and inject fake runners for backends.

**Global constraints (added):**
- Workflow/e2e tests use `typer.testing.CliRunner`; no `maeh.core` import in an
  e2e test body. Backend commands asserted via injected runner; git worktree via
  a `git init` temp repo. Coverage gate stays ≥90%.

---

## Task A — Config: worktree + workspace sections

**Files:** `src/maeh/core/config.py`, `tests/core/test_config.py`

**Interfaces:**
- `WorktreeConfig(prefix:str="maeh", location:str="~/.maeh/worktrees")`.
- `WorkspaceConfig(panes:dict[str,list[str]])` — `{"default": ["editor","primary","critic"], "<backend>": [...]}`; method `panes_for(backend)->list[str]` = override or `default`.
- `Config` gains `worktree: WorktreeConfig`, `workspace: WorkspaceConfig`.
- `load_config` parses `[worktree]` and `[workspace]` (+ `[workspace.<backend>]`).
- `config_to_dict` includes both.
- `resolve_worktree(cfg:WorktreeConfig, repo:Path, node_id:str)->tuple[Path,str]` — returns `(worktree_path, branch)`; central if location abs/`~`, else `<repo>/<location>/...`; `branch=f"{prefix}-{node_id}"`.

**Steps:** failing tests (defaults; `[workspace.tmux]` override; central vs `.worktrees` resolution) → implement → green → commit.

---

## Task B — `maeh default-config` + `config path`

**Files:** `src/maeh/core/config.py` (a `DEFAULT_CONFIG_TOML` template string), `src/maeh/cli/main.py`, `docs/config.example.toml` (regenerated to equal the template), `tests/cli/test_main.py`

**Interfaces:**
- `config.DEFAULT_CONFIG_TOML: str` — the commented template incl. `[worktree]`/`[workspace]`.
- `maeh default-config` → `typer.echo(DEFAULT_CONFIG_TOML)`.
- `maeh config path` → prints `resolve_home()/config.toml`.

**Steps:** CliRunner test asserts `default-config` output parses as TOML and `load_config` accepts it → implement → verify `docs/config.example.toml == DEFAULT_CONFIG_TOML` (a test) → commit.

---

## Task C — Workspace: worktree + panes, per backend

**Files:** `src/maeh/core/workspace.py`, `tests/core/test_workspace.py`

**Interfaces:**
- `WorkspaceHandle(node_id, backend, ref, worktree)`.
- `open_workspace(node:Node, config:Config, runner:Runner=_run)->WorkspaceHandle` (now takes full Config).
- `_role_cmds(config, backend)->list[str]` — `panes_for(backend)` mapped through `[agents]`, skipping empty.
- `_open_tmux(node, config, run)` — `git worktree add` (idempotent), `tmux new-session -A -d -s maeh`, window `maeh-<id>`, `split-window` ×(n−1) `-c <wt>`, `send-keys` per role.
- `_open_herdr(node, config, run)` — find-or-create by label; else `herdr worktree create --branch <b> --label maeh-<id> --cwd <repo>` → root pane; `pane run`; `pane split --target <prev> --direction vertical` + `pane run` per extra role.
- `SUPPORTED_BACKENDS` unchanged.

**Steps:** failing unit tests with a fake runner that returns canned JSON and records commands — assert (a) herdr sequence: list→worktree create→run→split→run; (b) tmux sequence: worktree add→new-session→split×→send-keys; (c) idempotent reuse when label/window exists; (d) git worktree add skipped when path exists (temp `git init` repo). → implement → green → commit.

**Ripple:** `cli/app.py` `on_tree_node_selected` calls `open_workspace(node, self._config)`.

---

## Task D — `maeh list --filter`

**Files:** `src/maeh/core/store.py`, `src/maeh/cli/main.py`, `tests/core/test_store.py`, `tests/cli/test_main.py`

**Interfaces:**
- `store.list_plans(home:Path)->list[dict]` — one attr map per `plans/*.json`: `{id, status(root), todo, running, done, failed}`. Empty list if no dir.
- `store.filter_plans(rows:list[dict], filters:dict[str,str])->list[dict]` — AND exact-match; raise `KeyError` on unknown key.
- `maeh list --filter k=v …` (repeatable) → `render(filtered_rows, output)`.

**Steps:** failing tests (list two saved plans; filter by `status`; unknown key raises) + CliRunner test (`maeh -o json list --filter status=todo`) → implement → green → commit.

---

## Task E — Plan CLI commands + `maeh open`

**Files:** `src/maeh/cli/main.py`, `tests/cli/test_main.py`

**Interfaces (Typer sub-app `plan` + top-level `open`):**
- `maeh plan create <id> <name>` → `save_plan(PlanTree(Node(id,name)), home)`.
- `maeh plan add <id> <node_id> <name> [--parent PID] [--path DIR]` → `update_plan(... add_child(parent or root, Node(...)))`.
- `maeh plan set-status <id> <node_id> <status>` → `update_plan(... set_status)`.
- `maeh open <id> <node_id>` → load, `open_workspace(node, config)`, `set_status RUNNING`, print handle. (Real backend — covered by an e2e test only when `MAEH_E2E=1`; unit path covered by Task C.)

**Steps:** CliRunner tests for create/add/set-status/list round-trip (no backend) → implement → green → commit. A CLI-only workflow test: create→add×→set-status→list asserting the tree — all via `CliRunner`, zero core imports for the actions.

---

## Task F — Docs + skills sync

**Files:** `docs/SPEC.md`, `README.md`, `skills/plan-to-workspaces/SKILL.md`, `docs/config.example.toml`

- SPEC §6/§8: add worktree/workspace config + new CLI commands; note panes.
- `plan-to-workspaces` skill: drive via `maeh open <id> <node>` (CLI), not core calls; mention worktree+panes.
- README: mention `maeh list`, `default-config`, worktree panes.

---

## Self-review checklist
- Every command respects `--set`/`-o`; `list`/`config`/`get`/`default-config` are read-only and pipeable.
- `open_workspace(node, config)` signature updated at all call sites (app.py, `maeh open`).
- herdr/tmux command sequences asserted by fake-runner tests; no live daemon in CI.
- `docs/config.example.toml` equals `DEFAULT_CONFIG_TOML` (enforced by a test).
