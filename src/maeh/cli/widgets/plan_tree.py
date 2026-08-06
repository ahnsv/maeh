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
            branch = parent.add(
                format_label(child, self._fmt),
                data=child,
                allow_expand=bool(child.children),
            )
            branch.expand()
            self._add(branch, child)
