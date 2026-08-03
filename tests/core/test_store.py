import pytest

from maeh.core.models import Node, PlanTree, Status
from maeh.core.plan import set_status
from maeh.core.store import load_plan, plan_to_dict, save_plan, update_plan


def test_round_trip(tmp_path):
    t = PlanTree(
        Node(
            "p1",
            "root",
            Status.RUNNING,
            path="services/api",
            children=[Node("c", "child", Status.DONE)],
        )
    )
    save_plan(t, tmp_path)
    got = load_plan("p1", tmp_path)
    assert got.root.children[0].status is Status.DONE
    assert got.root.name == "root"
    assert got.root.path == "services/api"


def test_load_plan_rejects_traversal(tmp_path):
    with pytest.raises(ValueError):
        load_plan("../../../etc/passwd", tmp_path)


def test_update_plan_locks_load_mutate_save(tmp_path):
    save_plan(PlanTree(Node("p1", "root")), tmp_path)
    update_plan(tmp_path, "p1", lambda t: set_status(t, "p1", Status.DONE))
    assert load_plan("p1", tmp_path).root.status is Status.DONE


def test_plan_to_dict(tmp_path):
    d = plan_to_dict(PlanTree(Node("p1", "root", Status.DONE)))
    assert d["id"] == "p1" and d["status"] == "done"
