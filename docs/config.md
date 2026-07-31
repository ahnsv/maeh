# maeh config

`maeh` uses two TOML configs:

1. User config: stores the default home path.
2. Home config: stores runtime behavior for the selected home.

## Home resolution

Precedence is highest to lowest:

1. `--home <path>`
2. `MAEH_HOME`
3. user config `[paths].home`
4. `$HOME/.maeh`
5. `.maeh` when `HOME` is unset

Persist a default home:

```bash
maeh --home ~/.claude/orchestrator config set-home
```

That writes the user config at `MAEH_CONFIG`, `$XDG_CONFIG_HOME/maeh/config.toml`, or `~/.config/maeh/config.toml`:

```toml
[paths]
home = "/absolute/path/to/orchestrator"
```

Relative paths are resolved from the user config file's directory. `~` and `~/...` expand when `HOME` is set.

## Home config

`maeh init` writes `<home>/config.toml`. See [`docs/config.example.toml`](config.example.toml) for a copyable sample.

```toml
[backend]
kind = "auto"
herdr_bin = "herdr"
tmux_bin = "tmux"
tmux_session = "maeh"

[layout]
include_editor = true
focus = false

[agents]
primary_cmd = "codex"
critic_cmd = "codex"
editor_cmd = "vi"

[limits]
context_switch_cap = 3
review_cap = 5

[board_cache]
intake_ttl_secs = 3600
revamp_ttl_secs = 10800

[task_capsules]
max_chars = 1800

[work_hours]
start_hour = 9
end_hour = 17
workdays = [1, 2, 3, 4, 5]
```

## Reference

### `[backend]`

| Key | Default | Meaning | Env override |
| --- | --- | --- | --- |
| `kind` | `"auto"` | Backend selection: `auto`, `herdr`, or `tmux`. | `MAEH_BACKEND` |
| `herdr_bin` | `"herdr"` | Herdr executable. | `MAEH_HERDR_BIN` |
| `tmux_bin` | `"tmux"` | tmux executable. | `MAEH_TMUX_BIN` |
| `tmux_session` | `"maeh"` | tmux session for managed windows. | `MAEH_TMUX_SESSION` |

`auto` selects Herdr when `HERDR_ENV` or `HERDR_SOCKET_PATH` is present; otherwise it selects tmux.

### `[layout]`

| Key | Default | Meaning | Env override |
| --- | --- | --- | --- |
| `include_editor` | `true` | Spawn an editor pane when a command supports one. | `MAEH_INCLUDE_EDITOR` |
| `focus` | `false` | Ask the backend to focus/open the created workspace. | `MAEH_FOCUS` |

### `[agents]`

| Key | Default | Meaning | Env override |
| --- | --- | --- | --- |
| `primary_cmd` | `"codex"` | Command used for the primary agent. | `MAEH_PRIMARY_AGENT_CMD` |
| `critic_cmd` | `"codex"` | Command used for the critic agent. | `MAEH_CRITIC_AGENT_CMD` |
| `editor_cmd` | `"vi"` | Command used for the editor pane. | `MAEH_EDITOR_CMD` |

### `[limits]`

| Key | Default | Meaning |
| --- | --- | --- |
| `context_switch_cap` | `3` | Active-work cap used by `maeh cap check` and `maeh statusline`. |
| `review_cap` | `5` | Review cap used by `maeh cap check` and `maeh statusline`. |

### `[board_cache]`

| Key | Default | Meaning |
| --- | --- | --- |
| `intake_ttl_secs` | `3600` | Freshness window for intake board cache reads. |
| `revamp_ttl_secs` | `10800` | Freshness window for revamp board cache reads. |

### `[task_capsules]`

| Key | Default | Meaning |
| --- | --- | --- |
| `max_chars` | `1800` | Maximum serialized task capsule size. |

### `[work_hours]`

| Key | Default | Meaning |
| --- | --- | --- |
| `start_hour` | `9` | Inclusive local start hour. |
| `end_hour` | `17` | Exclusive local end hour. |
| `workdays` | `[1, 2, 3, 4, 5]` | ISO weekday numbers, Monday = 1. |

## Migration

Flat keys from earlier configs still load. Prefer the grouped keys above for new edits.
