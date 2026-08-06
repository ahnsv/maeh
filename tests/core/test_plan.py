import pytest

from maeh.core.models import Node, PlanTree, Status
from maeh.core.plan import add_child, set_status


def test_set_status_updates_node():
    t = PlanTree(Node("r", "root"))
    set_status(t, "r", Status.DONE)
    assert t.root.status is Status.DONE


def test_set_status_unknown_id_raises():
    with pytest.raises(KeyError):
        set_status(PlanTree(Node("r", "root")), "x", Status.DONE)


def test_add_child_attaches():
    t = PlanTree(Node("r", "root"))
    add_child(t, "r", Node("c", "child"))
    assert t.find("c").name == "child"
