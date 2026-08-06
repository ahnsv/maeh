# Addendum 02 — Implementation Plan (task capsules)

Extends `docs/PLAN.md`. Determinism is the invariant: capsule rendering has no
clock/random and is covered by a golden test. Workflow paths tested via CliRunner.

**Import graph (no cycles):** `capsule` → {`config`, `store`, `models`};
`workspace` takes pre-rendered capsule paths as data (does **not** import
`capsule`). `config` → `workspace` (unchanged).

---

## Task A — `Node.brief` + `plan add --brief`

**Files:** `src/maeh/core/models.py`, `src/maeh/core/store.py`, `src/maeh/cli/main.py`, tests.

- `Node(id, name, status=TODO, path=None, brief=None, children=[])` — `brief` after
  `path`, before `children` (constructors already pass `children=` by keyword).
- `store._to_dict`/`_from_dict` round-trip `brief` (`d.get("brief")`).
- `maeh plan add … --brief TEXT` sets it.

**Steps:** failing round-trip test (brief persists); CliRunner `plan add --brief` +
`get` shows it → implement → green → commit.

---

## Task B — `core.capsule` pure render

**Files:** `src/maeh/core/capsule.py`, `tests/core/test_capsule.py`

**Interfaces:**
- `capsule(tree: PlanTree, node_id: str, role: str, guardrails: list[str]) -> str`.
- `_ancestry(tree, node_id) -> list[Node]` — root→node chain (BFS/DFS with parent
  tracking; raises `KeyError` if absent).
- Deterministic Markdown: `# <goal>` / `## Context` (ancestor names) / `## Task`
  (node name + brief or "(no brief)") / `## Guardrails` (list + conventions line) /
  role framing line. Role in `{primary, critic, editor}`; other → neutral.

**Steps:** golden test — build a small tree, assert `capsule(...)` equals an exact
expected string for `role=primary` and `role=critic`; assert no `datetime`/`random`
import in the module (ast scan, like the boundary test). → implement → green → commit.

---

## Task C — `maeh capsule` command

**Files:** `src/maeh/cli/main.py`, `tests/cli/test_main.py`

- `maeh capsule <plan_id> <node_id> [--role primary]` → loads plan + config, prints
  `render(capsule_text, output)`: `plaintext` = the Markdown, `json` =
  `{role, node_id, text}`.

**Steps:** CliRunner test: create plan, add node `--brief`, `maeh capsule wf n1
--role critic` contains the brief and the critic framing → implement → commit.

---

## Task D — Seed panes with `{capsule}` (deterministic)

**Files:** `src/maeh/core/store.py`, `src/maeh/core/capsule.py`, `src/maeh/core/workspace.py`, `src/maeh/cli/main.py`, `src/maeh/cli/app.py`, tests.

**Interfaces:**
- `store.write_capsule(home, plan_id, node_id, role, text) -> Path` — writes
  `<home>/capsules/<plan_id>/<node_id>-<role>.md` 0600 (ids validated, `private_subdir`).
- `capsule.prepare(tree, node_id, config, home) -> dict[str, str]` — for each role in
  `config.workspace.panes_for(config.backend)`, render + write, return `{role: abspath}`.
- `workspace._role_cmds(config) -> list[tuple[str, str]]` — now `(role, cmd)` pairs.
- `workspace.open_workspace(node, config, capsule_paths: dict[str,str] | None = None, runner=_run)` —
  for each `(role, cmd)`, `cmd.replace("{capsule}", capsule_paths[role])` when the
  role has a path and the placeholder is present; otherwise run unchanged.
- `maeh open` and `app.on_tree_node_selected`: `paths = capsule.prepare(tree, node_id, cfg, home)`; `open_workspace(node, cfg, paths)`.

**Steps:** fake-runner tests — with `primary_cmd="pi {capsule}"`, assert the tmux
`send-keys`/herdr `pane run` command for the primary pane is `pi <abs capsule
path>`, that the capsule file exists at the expected path, and that a command with
no `{capsule}` is unchanged. Update the existing pane-count tests to the `(role,
cmd)` shape. → implement → green → commit.

---

## Task E — `task-to-plan` skill populates `--brief`

**Files:** `skills/task-to-plan/SKILL.md`

- Step: after distilling each node's scope/acceptance, persist it with
  `maeh plan add … --brief "<scope + acceptance>"`. State that the brief is frozen
  at plan time and the capsule is a pure render (no re-cooking at open).

---

## Self-review
- `capsule()` is pure (golden + no-clock/random ast test).
- `open_workspace` signature updated at both call sites (`maeh open`, app).
- `{capsule}` substitution is per-role; no placeholder → command unchanged; file
  always written under `$MAEH_HOME` (never the repo).
- Coverage ≥90; ruff+format clean.
