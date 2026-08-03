from maeh.cli.widgets.plan_tree import format_label
from maeh.core.config import TuiConfig
from maeh.core.models import Node, Status


def test_format_label_uses_status_format():
    fmt = TuiConfig().status_format
    assert format_label(Node("i", "task", Status.DONE), fmt) == "[green]✔[/] task"


def test_format_label_unknown_status_falls_back():
    assert format_label(Node("i", "task", Status.TODO), {}) == "[white]?[/] task"
