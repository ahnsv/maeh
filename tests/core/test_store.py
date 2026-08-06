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


def test_list_and_filter_plans(tmp_path):
    from maeh.core.store import filter_plans, list_plans

    save_plan(
        PlanTree(
            Node("a", "A", Status.RUNNING, children=[Node("c", "C", Status.DONE)])
        ),
        tmp_path,
    )
    save_plan(PlanTree(Node("b", "B", Status.TODO)), tmp_path)
    rows = list_plans(tmp_path)
    assert {r["id"] for r in rows} == {"a", "b"}
    running = filter_plans(rows, {"status": "running"})
    assert [r["id"] for r in running] == ["a"]
    assert filter_plans(rows, {"done": "1"})[0]["id"] == "a"  # int stringified


def test_filter_unknown_key_raises_even_with_no_plans(tmp_path):
    from maeh.core.store import filter_plans, list_plans

    with pytest.raises(KeyError):
        filter_plans(list_plans(tmp_path), {"bogus": "1"})


def test_save_handle_is_private(tmp_path):
    import stat

    from maeh.core.store import save_handle

    p = save_handle(
        tmp_path,
        {"node_id": "n1", "backend": "tmux", "ref": "maeh-n1", "worktree": "/wt"},
    )
    assert stat.S_IMODE(p.stat().st_mode) == 0o600


def test_write_capsule_private_and_validates_role(tmp_path):
    import stat

    from maeh.core.store import write_capsule

    p = write_capsule(tmp_path, "plan1", "n1", "primary", "hello")
    assert p.read_text() == "hello"
    assert stat.S_IMODE(p.stat().st_mode) == 0o600
    with pytest.raises(ValueError):
        write_capsule(tmp_path, "plan1", "n1", "../evil", "x")  # role path-injection
