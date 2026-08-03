from __future__ import annotations

import os
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

from maeh.core.workspace import SUPPORTED_BACKENDS

_DEFAULT_STATUS_FORMAT = {
    "done": ("✔", "green"),
    "running": ("◐", "yellow"),
    "todo": ("○", "grey50"),
    "failed": ("✗", "red"),
}


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
    if cfg.backend not in SUPPORTED_BACKENDS:
        raise ValueError(
            f"backend {cfg.backend!r} not supported; "
            f"available: {sorted(SUPPORTED_BACKENDS)}"
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
        "agents": {
            "primary_cmd": cfg.agents.primary_cmd,
            "critic_cmd": cfg.agents.critic_cmd,
            "editor_cmd": cfg.agents.editor_cmd,
        },
        "tui": {
            "status_format": {k: list(v) for k, v in cfg.tui.status_format.items()}
        },
        "review": {"guardrails": cfg.review.guardrails},
        "limits": {"max_concurrent_workspaces": cfg.limits.max_concurrent_workspaces},
    }
