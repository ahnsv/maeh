from __future__ import annotations

from pathlib import Path

from textual.app import App, ComposeResult
from textual.widgets import Footer, Header, Tree

from maeh.cli.widgets.plan_tree import PlanTreeWidget
from maeh.core.config import Config
from maeh.core.models import PlanTree
from maeh.core.workspace import open_workspace


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
            handle = open_workspace(node, cwd, self._config.backend)
            self.notify(f"workspace {handle.ref}", title=node.name)
