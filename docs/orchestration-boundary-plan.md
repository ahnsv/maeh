# maeh orchestration boundary plan

## Assumptions

- `maeh` owns backend-aware slot state; skills may call `maeh` commands and use only tiny shell glue.
- Cleanup/revamp commands are reversible by default: plan/list/inspect first, mutate only explicit actions.
- Worktree removal/prune is planned by `maeh`; destructive removal still requires explicit command invocation by the caller.
- Skills repo changes land through the existing w16 workspace; this worktree adds the missing `maeh` contract and coordinates that contract to w16.

## Plan

1. Add backend-aware slot/status surfaces: list/inspect/reconcile output that cleanup/status/revamp can consume without parsing Herdr/tmux/git.
2. Add lifecycle actions for cleanup/revamp: classify done/stale/blocked, snooze/block/nudge/resume, close/remove slot, worktree prune/remove planning, and outcome summary.
3. Add cap/work-hours/state/ledger wrappers so skills can replace shell helpers with direct `maeh` calls.
4. Update docs/tests/README and coordinate the final command contract to w16 so skills shrink to `maeh` calls.
5. Verify with fmt, clippy, tests, coverage, critic review, and update PR #4.
