from __future__ import annotations

import typer

from maeh.cli.app import PlanApp
from maeh.cli.render import OutputFormat, render
from maeh.core.config import config_to_dict, load_config
from maeh.core.store import load_plan, plan_to_dict

app = typer.Typer(help="maeh — a maestro that orchestrates agents.")
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
    typer.echo(
        render(plan_to_dict(load_plan(plan_id, cfg.maeh_home)), _STATE["output"])
    )
