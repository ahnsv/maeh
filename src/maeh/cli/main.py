from __future__ import annotations

from datetime import UTC, datetime

import typer

from maeh.cli.app import PlanApp
from maeh.cli.render import OutputFormat, render
from maeh.core.config import DEFAULT_CONFIG_TOML, config_to_dict, load_config
from maeh.core.models import Node, PlanTree, Status
from maeh.core.plan import add_child, set_status
from maeh.core.store import (
    filter_plans,
    list_plans,
    load_plan,
    plan_to_dict,
    save_handle,
    save_plan,
    update_plan,
)
from maeh.core.telemetry import emit_metric, log
from maeh.core.workspace import open_workspace

app = typer.Typer(help="maeh — a maestro that orchestrates agents.")
plan_app = typer.Typer(help="Create and mutate plan trees.")
app.add_typer(plan_app, name="plan")

_STATE: dict = {"overrides": [], "output": OutputFormat.plaintext}


@app.callback()
def main(
    set_: list[str] = typer.Option(
        [],
        "--set",
        metavar="path.key=value",
        help="override a config value (repeatable, Helm-style)",
    ),
    output: OutputFormat = typer.Option(
        OutputFormat.plaintext, "-o", "--output", help="output format for read commands"
    ),
) -> None:
    _STATE["overrides"] = set_
    _STATE["output"] = output


def _config():
    return load_config(overrides=_STATE["overrides"])


@app.command()
def show(plan_id: str) -> None:
    """Open the plan tree TUI for PLAN_ID."""
    cfg = _config()
    PlanApp(load_plan(plan_id, cfg.maeh_home), cfg).run()


@app.command()
def config() -> None:
    """Print the effective config (respects --set and -o)."""
    typer.echo(render(config_to_dict(_config()), _STATE["output"]))


@app.command("default-config")
def default_config() -> None:
    """Print the default config.toml — scaffold with `> ~/.maeh/config.toml`."""
    typer.echo(DEFAULT_CONFIG_TOML, nl=False)


@app.command()
def get(plan_id: str) -> None:
    """Print a plan tree as data — pipe into jq/yq with -o json|yaml."""
    cfg = _config()
    typer.echo(
        render(plan_to_dict(load_plan(plan_id, cfg.maeh_home)), _STATE["output"])
    )


@app.command("list")
def list_cmd(
    filters: list[str] = typer.Option(
        [], "--filter", metavar="key=value", help="filter by key (repeatable, AND)"
    ),
) -> None:
    """List workflows (plan trees) with status counts."""
    cfg = _config()
    parsed: dict[str, str] = {}
    for f in filters:
        if "=" not in f:
            raise typer.BadParameter(f"--filter must be key=value: {f!r}")
        k, v = f.split("=", 1)
        parsed[k] = v
    try:
        rows = filter_plans(list_plans(cfg.maeh_home), parsed)
    except KeyError as e:
        raise typer.BadParameter(str(e)) from None
    typer.echo(render(rows, _STATE["output"]))


@plan_app.command("create")
def plan_create(plan_id: str, name: str) -> None:
    """Create and persist a root plan tree."""
    cfg = _config()
    save_plan(PlanTree(Node(plan_id, name)), cfg.maeh_home)
    typer.echo(f"created {plan_id}")


@plan_app.command("add")
def plan_add(
    plan_id: str,
    node_id: str,
    name: str,
    parent: str = typer.Option(None, "--parent", help="parent node id (default: root)"),
    path: str = typer.Option(
        None, "--path", help="code location for the node's workspace"
    ),
) -> None:
    """Add a node to a plan tree."""
    cfg = _config()
    update_plan(
        cfg.maeh_home,
        plan_id,
        lambda t: add_child(t, parent or t.root.id, Node(node_id, name, path=path)),
    )
    typer.echo(f"added {node_id}")


@plan_app.command("set-status")
def plan_set_status(plan_id: str, node_id: str, status: str) -> None:
    """Set a node's status (todo|running|done|failed)."""
    cfg = _config()
    st = Status(status)
    update_plan(cfg.maeh_home, plan_id, lambda t: set_status(t, node_id, st))
    typer.echo(f"{node_id} -> {st.value}")


@app.command("open")
def open_cmd(plan_id: str, node_id: str) -> None:
    """Execute: open the node's worktree-backed workspace and set it RUNNING."""
    cfg = _config()
    node = load_plan(plan_id, cfg.maeh_home).find(node_id)
    if node is None:
        raise typer.BadParameter(f"node {node_id!r} not in {plan_id!r}")
    handle = open_workspace(node, cfg)
    update_plan(
        cfg.maeh_home, plan_id, lambda t: set_status(t, node_id, Status.RUNNING)
    )
    record = {
        "node_id": handle.node_id,
        "backend": handle.backend,
        "ref": handle.ref,
        "worktree": handle.worktree,
    }
    save_handle(cfg.maeh_home, record)
    ts = datetime.now(UTC).isoformat()
    log(
        cfg.maeh_home,
        f"opened {handle.backend}:{handle.ref}",
        plan_id=plan_id,
        node_id=node_id,
        ts=ts,
        event="execute",
    )
    emit_metric(cfg.maeh_home, "workspaces_opened", record, ts=ts)
    typer.echo(render(record, _STATE["output"]))
