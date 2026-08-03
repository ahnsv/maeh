# maeh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the maeh v1 skeleton — a Python core holding all domain logic, a Textual CLI that is presentation-only, and the six workflow skills.

**Architecture:** Layered. `maeh.core` owns every domain rule (plan tree, config, persistence, workspace dispatch, telemetry) and imports neither `textual` nor `maeh.cli`. `maeh.cli` translates user input into `core` service calls and renders `core` values. Skills are plain markdown consumed by whatever AI harness runs them.

**Tech Stack:** Python ≥3.11, Textual (TUI), Typer (CLI parsing), stdlib `tomllib`/`json`/`pathlib`, pytest, ruff, uv. (Metrics are plain jsonl aggregated with stdlib `json`; no DuckDB dependency in v1.)

## Global Constraints

- Python ≥ 3.11 (uses stdlib `tomllib`). One line each, verbatim from SPEC:
- **CLI/TUI is presentation only** — no business logic in `maeh.cli`; all logic in `maeh.core`.
- **`maeh.core` must not import `maeh.cli` or `textual`** — enforced by a test.
- Config default location: `$MAEH_HOME/config.toml`, `$MAEH_HOME` defaults to `~/.maeh`.
- Every plan-tree node has a stable id; that id is the logging key and the workspace handle.
- Skills: `SKILL.md` frontmatter `name` + third-person `description` with trigger phrases; imperative body; progressive disclosure.
- No external DB in v1; state persists as files under `$MAEH_HOME`.
- Data at rest: `$MAEH_HOME` and subdirs are `0700`, content files `0600` (via `fsutil`).
- All plan-tree mutations go through `store.update_plan` (per-plan file lock) — never a bare `load`+`save`.
- CLI conventions: a global repeatable `--set path.key=value` (Helm-style, coerced + applied in `core.config`) and `-o/--output {json,yaml,plaintext}` on read commands.

## File Structure

```
pyproject.toml                     # package + deps + tool config
src/maeh/core/__init__.py          # public core API re-exports
src/maeh/core/models.py            # Status, Node, PlanTree, Increment
src/maeh/core/plan.py              # plan-tree construction/mutation helpers
src/maeh/core/config.py            # Config, TuiConfig, load_config()
src/maeh/core/fsutil.py            # private (0700/0600) dir + atomic write helpers
src/maeh/core/store.py             # save_plan()/load_plan()/update_plan() under $MAEH_HOME
src/maeh/core/workspace.py         # WorkspaceHandle, open_workspace() (tmux)
src/maeh/core/telemetry.py         # structured jsonl logs + metrics writer
src/maeh/cli/render.py             # OutputFormat + render() (json/yaml/plaintext)
src/maeh/cli/main.py               # Typer entrypoint; subcommands -> core
src/maeh/cli/app.py                # Textual App (presentation)
src/maeh/cli/widgets/plan_tree.py  # renders core.PlanTree; emits node actions
skills/<stage>/SKILL.md            # six skills
tests/...                          # one test module per core module + boundary
```

**Responsibilities:** `models` = data + id validation; `plan` = pure tree ops; `fsutil` = private/atomic filesystem primitives; `config`/`store` = IO at the `$MAEH_HOME` boundary (`store` also owns concurrency via a per-plan lock); `workspace` = side-effecting tmux dispatch (herdr added later, behind a protocol, only when it's real); `telemetry` = append-only structured logs/metrics; `cli/*` = collect `--set` overrides, render results in the chosen format, route input — zero rules.

---

## Phase 0 — Scaffold

### Task 0: Project layout, packaging, tooling

**Files:**
- Create: `pyproject.toml`, `src/maeh/__init__.py`, `src/maeh/core/__init__.py`, `src/maeh/cli/__init__.py`
- Modify: `.tool-versions` (rust → python 3.11), `.github/workflows/ci.yml` and `.github/workflows/release.yml` (Rust → uv/pytest/ruff + PyPI). Both are already rewritten in this repo — Task 0 just makes the package they assume exist.

**Interfaces:**
- Produces: an installable `maeh` package with `maeh` console script → `maeh.cli.main:app`.

- [ ] **Step 1: Write `pyproject.toml`**

```toml
[project]
name = "maeh"
version = "0.2.0"
requires-python = ">=3.11"
dependencies = ["textual>=0.60", "typer>=0.12", "pyyaml>=6"]

[project.scripts]
maeh = "maeh.cli.main:app"

[project.optional-dependencies]
dev = ["pytest>=8", "pytest-cov", "pytest-asyncio", "ruff"]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.pytest.ini_options]
pythonpath = ["src"]
testpaths = ["tests"]
asyncio_mode = "auto"
```

- [ ] **Step 2: CI** (`.github/workflows/ci.yml`) — on PR + push to main, `astral-sh/setup-uv`, matrix Python 3.11/3.12, `uv sync --all-extras`, then `ruff check`, `ruff format --check`, `pytest --cov=maeh --cov-fail-under=90`. `concurrency` cancels superseded runs; `permissions: contents: read`.
- [ ] **Step 3: Release** (`.github/workflows/release.yml`) — on tag `v*`: `build` job verifies the tag equals the `pyproject` version, runs tests, `uv build` (sdist+wheel); `pypi` job publishes via **PyPI Trusted Publishing (OIDC, `id-token: write`, `pypi` environment) — no API token**; `github-release` job attaches artifacts with generated notes. Distribution is `pip/uv/pipx install maeh`; standalone binaries (PyInstaller) are a later add if non-Python users need them.
- [ ] **Step 4: Set `.tool-versions` to `python 3.11.x`.**
- [ ] **Step 5: Commit `uv.lock`** and switch both workflows' `uv sync` to `--frozen` for reproducible installs.
- [ ] **Step 6: Verify** `uv sync && uv run pytest` collects 0 tests and exits 0.
- [ ] **Step 7: Commit** `chore: scaffold python project`.

---

## Phase 1 — Core domain

### Task 1: Models — Status, Node, PlanTree

**Files:**
- Create: `src/maeh/core/models.py`
- Test: `tests/core/test_models.py`

**Interfaces:**
- Produces: `require_safe_segment(value:str)->str` (returns `value` if it matches `^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9])?$` — alnum start/end, dots/dashes/underscores inside, so `..` and leading/trailing dots are rejected; else raises `ValueError`); `Status` (enum: `TODO`,`RUNNING`,`DONE`,`FAILED`, values `"todo"|"running"|"done"|"failed"`); `Node(id:str, name:str, status:Status=Status.TODO, path:str|None=None, children:list[Node]=[])` — `id` is validated in `__post_init__`; `path` is the code location the node's workspace opens in; `PlanTree(root:Node)` with `find(node_id:str)->Node|None` and `walk()->Iterator[Node]` (pre-order).

- [ ] **Step 1: Write the failing test**

```python
# tests/core/test_models.py
import pytest
from maeh.core.models import Status, Node, PlanTree, require_safe_segment

def _tree():
    return PlanTree(Node("r", "root", children=[
        Node("a", "a", Status.DONE),
        Node("b", "b", children=[Node("b1", "b1")]),
    ]))

def test_find_returns_node_by_id():
    assert _tree().find("b1").name == "b1"

def test_find_missing_returns_none():
    assert _tree().find("nope") is None

def test_walk_is_preorder():
    assert [n.id for n in _tree().walk()] == ["r", "a", "b", "b1"]

def test_node_id_rejects_path_traversal():
    with pytest.raises(ValueError):
        Node("../../etc/passwd", "evil")

def test_require_safe_segment_allows_dotted_ids():
    assert require_safe_segment("plan-1.2.1") == "plan-1.2.1"
```

- [ ] **Step 2: Run** `pytest tests/core/test_models.py -v` → FAIL (module missing).
- [ ] **Step 3: Implement**

```python
# src/maeh/core/models.py
from __future__ import annotations
import re
from collections.abc import Iterator
from dataclasses import dataclass, field
from enum import Enum

# Must start and end alphanumeric; dots/dashes/underscores allowed inside.
# Rejects "..", leading/trailing dots, slashes — safe as a filename segment.
_SAFE_SEGMENT = re.compile(r"^[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9])?$")

def require_safe_segment(value: str) -> str:
    if not _SAFE_SEGMENT.fullmatch(value):
        raise ValueError(f"unsafe identifier: {value!r}")
    return value

class Status(str, Enum):
    TODO = "todo"
    RUNNING = "running"
    DONE = "done"
    FAILED = "failed"

@dataclass
class Node:
    id: str
    name: str
    status: Status = Status.TODO
    path: str | None = None   # code location the node's workspace opens in
    children: list["Node"] = field(default_factory=list)

    def __post_init__(self) -> None:
        require_safe_segment(self.id)

@dataclass
class Increment:
    node_id: str
    kind: str          # "pr" | "document" | "artifact"
    ref: str           # URL / path / id

@dataclass
class PlanTree:
    root: Node

    def walk(self) -> Iterator[Node]:
        stack = [self.root]
        while stack:
            node = stack.pop()
            yield node
            stack.extend(reversed(node.children))  # pre-order, left-to-right

    def find(self, node_id: str) -> Node | None:
        return next((n for n in self.walk() if n.id == node_id), None)
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): plan-tree models`.

### Task 2: Plan operations — build/mutate

**Files:**
- Create: `src/maeh/core/plan.py`
- Test: `tests/core/test_plan.py`

**Interfaces:**
- Consumes: `Node`, `PlanTree`, `Status` from Task 1.
- Produces: `set_status(tree:PlanTree, node_id:str, status:Status)->None` (raises `KeyError` if id absent); `add_child(tree:PlanTree, parent_id:str, child:Node)->None` (raises `KeyError`).

- [ ] **Step 1: Failing test**

```python
# tests/core/test_plan.py
import pytest
from maeh.core.models import Node, PlanTree, Status
from maeh.core.plan import set_status, add_child

def test_set_status_updates_node():
    t = PlanTree(Node("r", "root"))
    set_status(t, "r", Status.DONE)
    assert t.root.status is Status.DONE

def test_set_status_unknown_id_raises():
    with pytest.raises(KeyError):
        set_status(PlanTree(Node("r", "root")), "x", Status.DONE)

def test_add_child_attaches():
    t = PlanTree(Node("r", "root"))
    add_child(t, "r", Node("c", "child"))
    assert t.find("c").name == "child"
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/core/plan.py
from maeh.core.models import Node, PlanTree, Status

def _require(tree: PlanTree, node_id: str) -> Node:
    node = tree.find(node_id)
    if node is None:
        raise KeyError(node_id)
    return node

def set_status(tree: PlanTree, node_id: str, status: Status) -> None:
    _require(tree, node_id).status = status

def add_child(tree: PlanTree, parent_id: str, child: Node) -> None:
    _require(tree, parent_id).children.append(child)
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): plan mutation helpers`.

---

## Phase 2 — Config & persistence

### Task 3: Config loader

**Files:**
- Create: `src/maeh/core/config.py`
- Test: `tests/core/test_config.py`

**Interfaces:**
- Produces: `AgentsConfig(primary_cmd:str, critic_cmd:str, editor_cmd:str)`; `TuiConfig(status_format: dict[str, tuple[str, str]])` (status value → `(icon, color)`); `ReviewConfig(guardrails: list[str])`; `LimitsConfig(max_concurrent_workspaces:int)`; `Config(maeh_home:Path, backend:str, agents:AgentsConfig, tui:TuiConfig, review:ReviewConfig, limits:LimitsConfig)`; `load_config(home: Path | None = None, overrides: list[str] | None = None) -> Config`; `config_to_dict(cfg:Config) -> dict` (toml-shaped, for `-o`). Resolution order for home: arg → `$MAEH_HOME` → `~/.maeh` (single home config; no separate user config in v1). Missing `config.toml` → all defaults. Config values override defaults key-by-key. `overrides` are Helm-style `path.key=value` strings, coerced (`true/false`→bool, int, float, `a,b`→list, else str) and deep-set into the parsed data before dataclass construction — so `--set core.backend=herdr` is validated too. `backend` is validated against `_IMPLEMENTED_BACKENDS = {"tmux"}`; any other value raises `ValueError` (honest knob — no silent fallback). See `docs/config.example.toml`.

- [ ] **Step 1: Failing test**

```python
# tests/core/test_config.py
import pytest
from maeh.core.config import load_config, config_to_dict

def test_defaults_when_no_file(tmp_path):
    cfg = load_config(tmp_path)
    assert cfg.backend == "tmux"
    assert cfg.agents.primary_cmd == "claude"
    assert cfg.tui.status_format["done"] == ("✔", "green")
    assert cfg.limits.max_concurrent_workspaces == 3

def test_file_overrides(tmp_path):
    (tmp_path / "config.toml").write_text(
        '[agents]\nprimary_cmd = "codex"\n'
        '[tui.status_format]\ndone = ["✓", "cyan"]\n'
        '[review]\nguardrails = ["g.md"]\n'
    )
    cfg = load_config(tmp_path)
    assert cfg.agents.primary_cmd == "codex"
    assert cfg.agents.critic_cmd == "claude"          # untouched default kept
    assert cfg.tui.status_format["done"] == ("✓", "cyan")
    assert cfg.tui.status_format["failed"] == ("✗", "red")  # untouched default kept
    assert cfg.review.guardrails == ["g.md"]

def test_unimplemented_backend_rejected(tmp_path):
    (tmp_path / "config.toml").write_text('[core]\nbackend = "herdr"\n')
    with pytest.raises(ValueError):
        load_config(tmp_path)

def test_set_overrides_apply_and_coerce(tmp_path):
    cfg = load_config(tmp_path, overrides=[
        "agents.primary_cmd=codex", "limits.max_concurrent_workspaces=5",
    ])
    assert cfg.agents.primary_cmd == "codex"          # no file needed
    assert cfg.limits.max_concurrent_workspaces == 5   # coerced to int

def test_set_override_backend_is_validated(tmp_path):
    with pytest.raises(ValueError):
        load_config(tmp_path, overrides=["core.backend=herdr"])

def test_config_to_dict_roundtrips_shape(tmp_path):
    d = config_to_dict(load_config(tmp_path))
    assert d["core"]["backend"] == "tmux"
    assert d["tui"]["status_format"]["done"] == ["✔", "green"]
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/core/config.py
from __future__ import annotations
import os, tomllib
from dataclasses import dataclass, field
from pathlib import Path

_DEFAULT_STATUS_FORMAT = {
    "done": ("✔", "green"),
    "running": ("◐", "yellow"),
    "todo": ("○", "grey50"),
    "failed": ("✗", "red"),
}
_IMPLEMENTED_BACKENDS = {"tmux"}   # add "herdr" when it lands (SPEC §10 Q4)

@dataclass
class AgentsConfig:
    primary_cmd: str = "claude"
    critic_cmd: str = "claude"
    editor_cmd: str = "nvim"

@dataclass
class TuiConfig:
    status_format: dict[str, tuple[str, str]] = field(
        default_factory=lambda: dict(_DEFAULT_STATUS_FORMAT)
    )

@dataclass
class ReviewConfig:
    guardrails: list[str] = field(default_factory=list)

@dataclass
class LimitsConfig:
    max_concurrent_workspaces: int = 3

@dataclass
class Config:
    maeh_home: Path
    backend: str = "tmux"
    agents: AgentsConfig = field(default_factory=AgentsConfig)
    tui: TuiConfig = field(default_factory=TuiConfig)
    review: ReviewConfig = field(default_factory=ReviewConfig)
    limits: LimitsConfig = field(default_factory=LimitsConfig)

def resolve_home(home: Path | None = None) -> Path:
    if home is not None:
        return home
    env = os.environ.get("MAEH_HOME")
    return Path(env) if env else Path.home() / ".maeh"

def _coerce(raw: str):
    low = raw.lower()
    if low in ("true", "false"):
        return low == "true"
    if "," in raw:
        return [_coerce(p) for p in raw.split(",")]
    for cast in (int, float):
        try:
            return cast(raw)
        except ValueError:
            pass
    return raw

def _apply_overrides(data: dict, overrides: list[str]) -> None:
    for item in overrides:
        if "=" not in item:
            raise ValueError(f"override must be path.key=value: {item!r}")
        path, raw = item.split("=", 1)
        node = data
        keys = path.split(".")
        for k in keys[:-1]:
            node = node.setdefault(k, {})
            if not isinstance(node, dict):
                raise ValueError(f"override path {path!r} hits a non-table value")
        node[keys[-1]] = _coerce(raw)

def load_config(home: Path | None = None, overrides: list[str] | None = None) -> Config:
    root = resolve_home(home)
    cfg = Config(maeh_home=root)
    path = root / "config.toml"
    data = tomllib.loads(path.read_text()) if path.exists() else {}
    if overrides:
        _apply_overrides(data, overrides)

    cfg.backend = data.get("core", {}).get("backend", cfg.backend)
    if cfg.backend not in _IMPLEMENTED_BACKENDS:
        raise ValueError(
            f"backend {cfg.backend!r} not implemented; available: {sorted(_IMPLEMENTED_BACKENDS)}"
        )

    agents = data.get("agents", {})
    cfg.agents = AgentsConfig(
        primary_cmd=agents.get("primary_cmd", cfg.agents.primary_cmd),
        critic_cmd=agents.get("critic_cmd", cfg.agents.critic_cmd),
        editor_cmd=agents.get("editor_cmd", cfg.agents.editor_cmd),
    )

    for status, pair in data.get("tui", {}).get("status_format", {}).items():
        cfg.tui.status_format[status] = tuple(pair)

    cfg.review = ReviewConfig(
        guardrails=list(data.get("review", {}).get("guardrails", []))
    )
    cfg.limits = LimitsConfig(
        max_concurrent_workspaces=data.get("limits", {}).get(
            "max_concurrent_workspaces", cfg.limits.max_concurrent_workspaces
        )
    )
    return cfg

def config_to_dict(cfg: Config) -> dict:
    return {
        "core": {"backend": cfg.backend},
        "agents": {"primary_cmd": cfg.agents.primary_cmd,
                   "critic_cmd": cfg.agents.critic_cmd,
                   "editor_cmd": cfg.agents.editor_cmd},
        "tui": {"status_format": {k: list(v) for k, v in cfg.tui.status_format.items()}},
        "review": {"guardrails": cfg.review.guardrails},
        "limits": {"max_concurrent_workspaces": cfg.limits.max_concurrent_workspaces},
    }
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): config loader`.

### Task 4: fsutil + plan-tree persistence

**Files:**
- Create: `src/maeh/core/fsutil.py`, `src/maeh/core/store.py`
- Test: `tests/core/test_fsutil.py`, `tests/core/test_store.py`

**Interfaces:**
- `fsutil` produces: `private_subdir(home:Path, *parts:str)->Path` (creates each level `0700`, returns the leaf); `write_private(path:Path, text:str)->None` (atomic `0600` write via `os.open`+`os.replace`); `append_private(path:Path, line:str)->None` (`0600` append). Unix perms — fine on the darwin/linux target.
- `store` consumes: `PlanTree`, `Node`, `Status`, `require_safe_segment`, `fsutil`; `plan` (`set_status` etc. for callers of `update_plan`).
- `store` produces: `save_plan(tree:PlanTree, home:Path)->Path` (atomic `0600` write of `<home>/plans/<root.id>.json` under `0700` dirs); `load_plan(plan_id:str, home:Path)->PlanTree` (validates `plan_id`, raises `FileNotFoundError` if absent); `update_plan(home:Path, plan_id:str, mutate:Callable[[PlanTree],None])->PlanTree` (holds an exclusive per-plan `flock`, then load→mutate→save — the single writer all mutations route through); `plan_to_dict(tree:PlanTree)->dict` (for `-o`). Round-trip preserves ids, names, status, `path`, nesting.

- [ ] **Step 1: Failing test**

```python
# tests/core/test_fsutil.py
import stat
from maeh.core.fsutil import private_subdir, write_private

def test_private_subdir_is_0700(tmp_path):
    d = private_subdir(tmp_path, "plans")
    assert stat.S_IMODE((tmp_path / "plans").stat().st_mode) == 0o700

def test_write_private_is_0600_and_atomic(tmp_path):
    p = tmp_path / "f.json"
    write_private(p, "hello")
    assert p.read_text() == "hello"
    assert stat.S_IMODE(p.stat().st_mode) == 0o600
```

```python
# tests/core/test_store.py
import pytest
from maeh.core.models import Node, PlanTree, Status
from maeh.core.plan import set_status
from maeh.core.store import save_plan, load_plan, update_plan, plan_to_dict

def test_round_trip(tmp_path):
    t = PlanTree(Node("p1", "root", Status.RUNNING, path="services/api",
                      children=[Node("c", "child", Status.DONE)]))
    save_plan(t, tmp_path)
    got = load_plan("p1", tmp_path)
    assert got.root.children[0].status is Status.DONE
    assert got.root.name == "root"
    assert got.root.path == "services/api"

def test_load_plan_rejects_traversal(tmp_path):
    with pytest.raises(ValueError):
        load_plan("../../../etc/passwd", tmp_path)

def test_update_plan_locks_load_mutate_save(tmp_path):
    save_plan(PlanTree(Node("p1", "root")), tmp_path)
    update_plan(tmp_path, "p1", lambda t: set_status(t, "p1", Status.DONE))
    assert load_plan("p1", tmp_path).root.status is Status.DONE

def test_plan_to_dict(tmp_path):
    d = plan_to_dict(PlanTree(Node("p1", "root", Status.DONE)))
    assert d["id"] == "p1" and d["status"] == "done"
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/core/fsutil.py
from __future__ import annotations
import os
from pathlib import Path

def private_subdir(home: Path, *parts: str) -> Path:
    d = home
    d.mkdir(parents=True, exist_ok=True)
    d.chmod(0o700)
    for part in parts:
        d = d / part
        d.mkdir(exist_ok=True)
        d.chmod(0o700)
    return d

def write_private(path: Path, text: str) -> None:
    tmp = path.with_name(path.name + ".tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(text)
    os.replace(tmp, path)          # atomic; old file intact on crash

def append_private(path: Path, line: str) -> None:
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(line)
```

```python
# src/maeh/core/store.py
from __future__ import annotations
import fcntl, json, os
from collections.abc import Callable
from contextlib import contextmanager
from pathlib import Path
from maeh.core.fsutil import private_subdir, write_private
from maeh.core.models import Node, PlanTree, Status, require_safe_segment

def _to_dict(node: Node) -> dict:
    return {"id": node.id, "name": node.name, "status": node.status.value,
            "path": node.path, "children": [_to_dict(c) for c in node.children]}

def _from_dict(d: dict) -> Node:
    return Node(id=d["id"], name=d["name"], status=Status(d["status"]),
                path=d.get("path"), children=[_from_dict(c) for c in d["children"]])

def plan_to_dict(tree: PlanTree) -> dict:
    return _to_dict(tree.root)

def save_plan(tree: PlanTree, home: Path) -> Path:
    plans = private_subdir(home, "plans")
    path = plans / f"{tree.root.id}.json"          # root.id validated in __post_init__
    write_private(path, json.dumps(_to_dict(tree.root), ensure_ascii=False, indent=2))
    return path

def load_plan(plan_id: str, home: Path) -> PlanTree:
    require_safe_segment(plan_id)
    path = home / "plans" / f"{plan_id}.json"
    return PlanTree(_from_dict(json.loads(path.read_text())))

@contextmanager
def _plan_lock(home: Path, plan_id: str):
    require_safe_segment(plan_id)
    plans = private_subdir(home, "plans")
    fd = os.open(plans / f"{plan_id}.lock", os.O_WRONLY | os.O_CREAT, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)

def update_plan(home: Path, plan_id: str, mutate: Callable[[PlanTree], None]) -> PlanTree:
    """The single writer: lock, load, mutate, atomically save. All plan
    mutations route through here so concurrent primary/critic can't lose updates."""
    with _plan_lock(home, plan_id):
        tree = load_plan(plan_id, home)
        mutate(tree)
        save_plan(tree, home)
        return tree
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): plan persistence`.

---

## Phase 3 — Workspace backends

### Task 5: Workspace (tmux)

Only tmux is real in v1, so there is no backend protocol or registry — one direct
function. Add a `WorkspaceBackend` protocol and a dispatch dict only when `herdr`
is actually implemented (SPEC §10 Q4).

**Files:**
- Create: `src/maeh/core/workspace.py`
- Test: `tests/core/test_workspace.py`

**Interfaces:**
- Consumes: `Node`.
- Produces: `WorkspaceHandle(node_id:str, backend:str, ref:str)`; `open_workspace(node:Node, cwd:Path, runner:Callable[[list[str]],None]=_run)->WorkspaceHandle`. Runs `tmux new-session -A -d -s maeh-<node.id> -c <cwd>` via an injected runner (`-A` = create-or-reattach, so re-running after a crash is idempotent). `node.id` is already validated, so it is safe as a session name/argument.

- [ ] **Step 1: Failing test** (inject a fake runner — no real tmux in CI)

```python
# tests/core/test_workspace.py
from pathlib import Path
from maeh.core.models import Node
from maeh.core.workspace import open_workspace, WorkspaceHandle

def test_open_workspace_creates_session_named_by_node_id():
    calls = []
    h = open_workspace(Node("n1", "task"), Path("/tmp"), runner=calls.append)
    assert isinstance(h, WorkspaceHandle)
    assert h.ref == "maeh-n1"
    assert calls == [["tmux", "new-session", "-A", "-d", "-s", "maeh-n1", "-c", "/tmp"]]
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/core/workspace.py
from __future__ import annotations
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from maeh.core.models import Node

@dataclass(frozen=True)
class WorkspaceHandle:
    node_id: str
    backend: str
    ref: str

def _run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=True)

def open_workspace(
    node: Node, cwd: Path, runner: Callable[[list[str]], None] = _run
) -> WorkspaceHandle:
    session = f"maeh-{node.id}"
    runner(["tmux", "new-session", "-A", "-d", "-s", session, "-c", str(cwd)])
    return WorkspaceHandle(node.id, "tmux", session)
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): tmux workspace`.

---

## Phase 4 — Telemetry

### Task 6: Logs + jsonl metrics

**Files:**
- Create: `src/maeh/core/telemetry.py`
- Test: `tests/core/test_telemetry.py`

**Interfaces:**
- Produces: `log(home:Path, message:str, *, plan_id:str, node_id:str, ts:str, level:str="info", event:str="log")->None` (appends a JSON record `{ts, level, event, plan_id, node_id, message}` to `<home>/logs/<day>.jsonl`); `emit_metric(home:Path, name:str, value:dict, *, ts:str)->None` (appends `{ts, **value}` to `<home>/metrics/<name>/<day>.jsonl`). `ts` is an injected ISO-8601 string (callers control the clock → deterministic tests); `day = ts[:10]`, so a run crossing midnight no longer splits mid-record vs its own timestamp. Structured records let `improve-the-workflow` order events and filter by `event`/`level`. Files are written `0600` under `0700` dirs via `fsutil`.

- [ ] **Step 1: Failing test**

```python
# tests/core/test_telemetry.py
import json
import pytest
from maeh.core.telemetry import log, emit_metric

TS = "2026-08-03T09:15:00Z"

def test_log_writes_structured_record(tmp_path):
    log(tmp_path, "started", plan_id="p1", node_id="n1", ts=TS, event="execute")
    rec = json.loads((tmp_path / "logs" / "2026-08-03.jsonl").read_text().strip())
    assert rec == {"ts": TS, "level": "info", "event": "execute",
                   "plan_id": "p1", "node_id": "n1", "message": "started"}

def test_log_record_survives_newlines_in_message(tmp_path):
    log(tmp_path, "a\nfake forged", plan_id="p1", node_id="n1", ts=TS)
    lines = (tmp_path / "logs" / "2026-08-03.jsonl").read_text().splitlines()
    assert len(lines) == 1 and json.loads(lines[0])["message"] == "a\nfake forged"

def test_emit_metric_appends_jsonl(tmp_path):
    emit_metric(tmp_path, "tokens", {"n": 42}, ts=TS)
    emit_metric(tmp_path, "tokens", {"n": 7}, ts=TS)
    lines = (tmp_path / "metrics" / "tokens" / "2026-08-03.jsonl").read_text().splitlines()
    assert [json.loads(l)["n"] for l in lines] == [42, 7]

def test_emit_metric_rejects_unsafe_name(tmp_path):
    with pytest.raises(ValueError):
        emit_metric(tmp_path, "../evil", {"n": 1}, ts=TS)
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/core/telemetry.py
from __future__ import annotations
import json
from pathlib import Path
from maeh.core.fsutil import append_private, private_subdir
from maeh.core.models import require_safe_segment

def log(home: Path, message: str, *, plan_id: str, node_id: str,
        ts: str, level: str = "info", event: str = "log") -> None:
    d = private_subdir(home, "logs")
    rec = {"ts": ts, "level": level, "event": event,
           "plan_id": plan_id, "node_id": node_id, "message": message}
    append_private(d / f"{ts[:10]}.jsonl", json.dumps(rec, ensure_ascii=False) + "\n")

def emit_metric(home: Path, name: str, value: dict, *, ts: str) -> None:
    require_safe_segment(name)             # name is interpolated into the path
    d = private_subdir(home, "metrics", name)
    append_private(d / f"{ts[:10]}.jsonl", json.dumps({"ts": ts, **value}, ensure_ascii=False) + "\n")
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(core): telemetry`.

---

## Phase 5 — CLI (presentation only)

### Task 7: Plan-tree widget

**Files:**
- Create: `src/maeh/cli/widgets/plan_tree.py`, `src/maeh/cli/widgets/__init__.py`
- Test: `tests/cli/test_plan_tree_widget.py`

**Interfaces:**
- Consumes: `PlanTree`, `Node`, `Status`; `TuiConfig.status_format`.
- Produces: `format_label(node:Node, status_format:dict)->str` (pure; returns Rich markup `"[color]icon[/] name"`); `PlanTreeWidget(tree:PlanTree, status_format:dict)` — a `textual.widgets.Tree` subclass that builds nodes via `format_label` and stores each `Node` as `data`. **Only rendering — no domain logic.**

- [ ] **Step 1: Failing test** (test the pure formatter; the widget is thin)

```python
# tests/cli/test_plan_tree_widget.py
from maeh.core.models import Node, Status
from maeh.core.config import TuiConfig
from maeh.cli.widgets.plan_tree import format_label

def test_format_label_uses_status_format():
    fmt = TuiConfig().status_format
    assert format_label(Node("i", "task", Status.DONE), fmt) == "[green]✔[/] task"

def test_format_label_unknown_status_falls_back():
    assert format_label(Node("i", "task", Status.TODO), {}) == "[white]?[/] task"
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/cli/widgets/plan_tree.py
from __future__ import annotations
from rich.markup import escape
from textual.widgets import Tree
from maeh.core.models import Node, PlanTree

def format_label(node: Node, status_format: dict[str, tuple[str, str]]) -> str:
    icon, color = status_format.get(node.status.value, ("?", "white"))
    return f"[{color}]{icon}[/] {escape(node.name)}"

class PlanTreeWidget(Tree):
    def __init__(self, tree: PlanTree, status_format: dict[str, tuple[str, str]]):
        self._fmt = status_format
        super().__init__(format_label(tree.root, status_format), data=tree.root)
        self._plan = tree

    def on_mount(self) -> None:
        self.root.expand()
        self._add(self.root, self._plan.root)

    def _add(self, parent, node: Node) -> None:
        for child in node.children:
            branch = parent.add(format_label(child, self._fmt), data=child,
                                allow_expand=bool(child.children))
            branch.expand()
            self._add(branch, child)
```

- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** `feat(cli): plan-tree widget`.

### Task 8: Textual app + Typer entrypoint (with `--set` / `-o`)

**Files:**
- Create: `src/maeh/cli/app.py`, `src/maeh/cli/main.py`, `src/maeh/cli/render.py`
- Test: `tests/cli/test_app.py` (Textual `App.run_test()` pilot), `tests/cli/test_render.py`

**Interfaces:**
- Consumes: `load_config`, `config_to_dict`, `load_plan`, `plan_to_dict`, `open_workspace`, `PlanTreeWidget`.
- Produces: `render.OutputFormat` (`json`/`yaml`/`plaintext`); `render.render(data, fmt)->str` (json via stdlib, yaml via `pyyaml`, plaintext = flattened `a.b.c = value` lines) — **presentation only**. `PlanApp(tree:PlanTree, config:Config)` — mounts `PlanTreeWidget`; on `Tree.NodeSelected`, calls `open_workspace(node, cwd)` where `cwd` is `node.path` (falling back to the process cwd). Typer `app` with a global callback exposing repeatable `--set path.key=value` and `-o/--output`, forwarded into `load_config(overrides=...)`; commands: `show <plan_id>` (TUI), `config` (prints the effective config), `get <plan_id>` (prints the plan tree) — the last two render via the chosen format so they pipe into `jq`/`yq`. The CLI holds no rules: it collects `--set` strings, calls core, and renders.

- [ ] **Step 1: Failing test**

```python
# tests/cli/test_app.py
import pytest
from maeh.core.models import Node, PlanTree
from maeh.core.config import Config
from maeh.cli.app import PlanApp

@pytest.mark.asyncio
async def test_app_mounts_tree(tmp_path):
    app = PlanApp(PlanTree(Node("r", "root")), Config(maeh_home=tmp_path))
    async with app.run_test() as pilot:
        await pilot.pause()
        from maeh.cli.widgets.plan_tree import PlanTreeWidget
        assert app.query_one(PlanTreeWidget) is not None
```

- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement**

```python
# src/maeh/cli/app.py
from __future__ import annotations
from pathlib import Path
from textual.app import App, ComposeResult
from textual.widgets import Footer, Header, Tree
from maeh.core.config import Config
from maeh.core.models import PlanTree
from maeh.core.workspace import open_workspace
from maeh.cli.widgets.plan_tree import PlanTreeWidget

class PlanApp(App):
    BINDINGS = [("q", "quit", "quit")]

    def __init__(self, tree: PlanTree, config: Config) -> None:
        super().__init__()
        self._tree = tree
        self._config = config

    def compose(self) -> ComposeResult:
        yield Header()
        yield PlanTreeWidget(self._tree, self._config.tui.status_format)
        yield Footer()

    def on_tree_node_selected(self, event: Tree.NodeSelected) -> None:
        node = event.node.data
        if node is not None:
            cwd = Path(node.path) if node.path else Path.cwd()
            handle = open_workspace(node, cwd)
            self.notify(f"workspace {handle.ref}", title=node.name)
```

```python
# src/maeh/cli/render.py
from __future__ import annotations
import json
from enum import Enum

class OutputFormat(str, Enum):
    json = "json"
    yaml = "yaml"
    plaintext = "plaintext"

def _flatten(data, prefix: str = "") -> list[str]:
    out: list[str] = []
    if isinstance(data, dict):
        for k, v in data.items():
            out += _flatten(v, f"{prefix}{k}.")
    elif isinstance(data, list):
        for i, v in enumerate(data):
            out += _flatten(v, f"{prefix}{i}.")
    else:
        out.append(f"{prefix.rstrip('.')} = {data}")
    return out

def render(data, fmt: OutputFormat) -> str:
    if fmt is OutputFormat.json:
        return json.dumps(data, ensure_ascii=False, indent=2)
    if fmt is OutputFormat.yaml:
        import yaml
        return yaml.safe_dump(data, allow_unicode=True, sort_keys=False).rstrip()
    return "\n".join(_flatten(data))
```

```python
# src/maeh/cli/main.py
from __future__ import annotations
from pathlib import Path
import typer
from maeh.core.config import load_config, config_to_dict
from maeh.core.store import load_plan, plan_to_dict
from maeh.cli.app import PlanApp
from maeh.cli.render import OutputFormat, render

app = typer.Typer(help="maeh — a maestro that orchestrates agents.")
_STATE: dict = {"overrides": [], "output": OutputFormat.plaintext}

@app.callback()
def main(
    set_: list[str] = typer.Option(
        [], "--set", metavar="path.key=value",
        help="override a config value (repeatable, Helm-style)"),
    output: OutputFormat = typer.Option(
        OutputFormat.plaintext, "-o", "--output", help="output format for read commands"),
) -> None:
    _STATE["overrides"] = set_
    _STATE["output"] = output

@app.command()
def show(plan_id: str) -> None:
    """Open the plan tree TUI for PLAN_ID."""
    config = load_config(overrides=_STATE["overrides"])
    PlanApp(load_plan(plan_id, config.maeh_home), config).run()

@app.command()
def config() -> None:
    """Print the effective config (respects --set and -o)."""
    cfg = load_config(overrides=_STATE["overrides"])
    typer.echo(render(config_to_dict(cfg), _STATE["output"]))

@app.command()
def get(plan_id: str) -> None:
    """Print a plan tree as data — pipe into jq/yq with -o json|yaml."""
    cfg = load_config(overrides=_STATE["overrides"])
    typer.echo(render(plan_to_dict(load_plan(plan_id, cfg.maeh_home)), _STATE["output"]))
```

- [ ] **Step 3b: Render test**

```python
# tests/cli/test_render.py
from maeh.cli.render import OutputFormat, render

def test_json(): 
    assert render({"a": 1}, OutputFormat.json) == '{\n  "a": 1\n}'

def test_plaintext_flattens():
    assert render({"a": {"b": 1}}, OutputFormat.plaintext) == "a.b = 1"

def test_yaml():
    assert render({"a": 1}, OutputFormat.yaml) == "a: 1"
```

- [ ] **Step 4: Run** → PASS (`pytest-asyncio` + `asyncio_mode = "auto"` from Task 0 make the async app test actually execute).
- [ ] **Step 5: Commit** `feat(cli): textual app, entrypoint, --set/-o read commands`.

---

## Phase 6 — Architecture boundary

### Task 9: Enforce core↔cli boundary

**Files:**
- Create: `tests/test_architecture.py`

**Interfaces:**
- Consumes: nothing runtime; statically scans `src/maeh/core`.

- [ ] **Step 1: Write the test**

```python
# tests/test_architecture.py
import ast, pathlib

CORE = pathlib.Path("src/maeh/core")

def _imports(path):
    tree = ast.parse(path.read_text())
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module:
            yield node.module
        if isinstance(node, ast.Import):
            for n in node.names:
                yield n.name

def test_core_never_imports_cli_or_textual():
    offenders = []
    for py in CORE.rglob("*.py"):
        for mod in _imports(py):
            if mod.startswith(("maeh.cli", "textual")):
                offenders.append((str(py), mod))
    assert offenders == [], offenders
```

- [ ] **Step 2: Run** → PASS (green if Phases 1-4 respected the rule).
- [ ] **Step 3: Commit** `test: enforce core/cli boundary`.

---

## Phase 7 — Skills

### Task 10: Author the six skills

**Files:**
- Modify: `skills/task-to-plan/SKILL.md`, `skills/plan-to-workspaces/SKILL.md`, `skills/review-the-increments/SKILL.md`, `skills/change-of-plan/SKILL.md`, `skills/move-to-gate/SKILL.md`, `skills/improve-the-workflow/SKILL.md`

**Interfaces:**
- Each `SKILL.md` has frontmatter `name` + third-person `description` with trigger phrases, and an imperative body. Detailed procedures move to `references/` per progressive disclosure. No tests (markdown).

- [ ] **Step 1:** For each skill, write frontmatter. Template:

```markdown
---
name: <stage-name>
description: This skill should be used when the user asks to "<phrase 1>", "<phrase 2>", or "<phrase 3>". <One line on what it does>.
version: 0.1.0
---
```

- [ ] **Step 2:** Write each body in imperative form covering: inputs it reads (plan tree, config, telemetry), the procedure, and the increment/output it produces. Per-skill focus:
  - `task-to-plan`: interview + repo exploration → emit a `PlanTree` (saved via `save_plan`).
  - `plan-to-workspaces`: map each node to a code location + backend; call `open_workspace`.
  - `review-the-increments`: check increments vs per-repo guidelines + guardrails; emit a verdict.
  - `change-of-plan`: on failure, mutate the plan tree (`set_status`/`add_child`) or route back to Execute.
  - `move-to-gate`: build the one-page gate summary (context, what-to-check, asks, changes).
  - `improve-the-workflow`: read logs/metrics/retro/improvement notes → one improvement proposal with goal, evidence, success criteria; revisit prior note.
- [ ] **Step 3:** Move any long procedure (>~2k words) into that skill's `references/`.
- [ ] **Step 4: Commit** `feat(skills): author v1 workflow skills`.

---

## Self-Review

- **Spec coverage:** SPEC §4 workflow → skills (Task 10); §5 plan tree → Tasks 1-2; §6 components → config/store/telemetry (Tasks 3,4,6); §7 core/cli rule → Task 9; §8 layout → File Structure; §9 skills → Task 10. Gate interactivity (SPEC open Q2) and token-budget metric (Q1) are **not** built here — they remain open questions, not silently implemented.
- **Type consistency:** `Status` values, `Node(id, name, status, path, children)` (constructors pass `children=` by keyword since `path` precedes it), `PlanTree`/`WorkspaceHandle`, `open_workspace(node, cwd, runner=_run)`, `require_safe_segment`, `format_label(node, status_format)`, `fsutil.private_subdir/write_private/append_private`, `store.save_plan/load_plan/update_plan/plan_to_dict`, `config.load_config(home, overrides)`/`config_to_dict`, `telemetry.log(..., ts, level, event)`/`emit_metric(..., ts)`, and `render.render(data, fmt)` are used identically across tasks.
- **Placeholders:** none — every code/test step carries runnable content. Skill bodies (Task 10) are described by required sections rather than final prose, since content depends on per-skill domain drafting.

## Execution Handoff

Plan saved to `docs/PLAN.md`. Execution options: (1) subagent-driven (fresh subagent per task, review between); (2) inline with checkpoints. Not started — this is a design deliverable pending review.
