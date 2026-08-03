import pytest

from maeh.core.models import Node, PlanTree, Status, require_safe_segment


def _tree():
    return PlanTree(
        Node(
            "r",
            "root",
            children=[
                Node("a", "a", Status.DONE),
                Node("b", "b", children=[Node("b1", "b1")]),
            ],
        )
    )


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
