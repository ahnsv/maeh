# Addendum 04 — Implementation Plan (`maeh init` + bundled data)

Extends `docs/PLAN.md`. CLI-driven tests, coverage ≥90, ruff+format clean.

---

## Task A — Bundle data + packaging

**Files:** move `guardrails/default.md` → `src/maeh/data/guardrails/default.md`;
`agents/orchestrator/AGENT.md` → `src/maeh/data/agents/orchestrator/AGENT.md`; delete
the top-level copies; `pyproject.toml`.

- Ensure the wheel ships `src/maeh/data/**` (hatchling includes package files by
  default; add an explicit `force-include`/`artifacts` only if a `uv build` check shows
  they're missing).

**Steps:** move files; `uv build`; unzip the wheel and confirm both `.md` files are
present under `maeh/data/…`.

---

## Task B — `core.scaffold.init_home`

**Files:** `src/maeh/core/scaffold.py`, `tests/core/test_scaffold.py`

**Interfaces:**
- `init_home(home: Path, *, force: bool = False) -> dict[str, str]`.
- Reads templates via `importlib.resources.files("maeh").joinpath("data", …)`.
- Uses `fsutil.private_subdir` (0700 dirs) + `fsutil.write_private` (0600 files).
- Config: `DEFAULT_CONFIG_TOML.replace('guardrails = []', f'guardrails = ["{guard_abs}"]')`.
- Existing file → `"skipped"` unless `force`; returns `{path: status}`.

**Steps:** failing tests →
1. into `tmp_path`: creates `config.toml`, `guardrails/default.md`,
   `agents/orchestrator/AGENT.md`; files are `0600`, dirs `0700`.
2. `load_config(tmp_path)` on the written config → `backend == "tmux"` and
   `review.guardrails == [str(tmp_path/"guardrails/default.md")]` (pre-wired + resolves).
3. second call → all `"skipped"`; `force=True` → all `"written"`.
4. `init` does not create `plans/` etc.
→ implement → green → commit.

---

## Task C — `maeh init` command

**Files:** `src/maeh/cli/main.py`, `tests/cli/test_main.py`

**Interfaces:**
- `maeh init [--force]` → `resolve_home()` (honors `$MAEH_HOME`), `init_home(...)`,
  echo `f"{status}: {path}"` per entry.

**Steps:** CliRunner test — `MAEH_HOME` set to tmp, `maeh init` exits 0 and the three
files exist; `maeh config` on the initialized home shows the guardrail wired; second
`maeh init` prints `skipped`. → implement → commit.

---

## Validation (live)
- `uv run maeh init --home <tmp>` for real → show the three files, `maeh config` reflects
  the wired guardrail, and `maeh capsule` (on a throwaway plan) lists the guardrail path.
- `uv build` → confirm `maeh/data/**` in the wheel (data actually ships).

---

## Self-review
- `init_home` is core logic; `maeh init` is a thin wrapper (CLI-is-presentation holds).
- Idempotent + never clobbers user config/state; `--force` is the only overwrite path.
- Guardrail path written absolute so the capsule reference resolves for working agents.
- Package-data inclusion verified by an actual wheel build, not assumed.
