from __future__ import annotations

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
