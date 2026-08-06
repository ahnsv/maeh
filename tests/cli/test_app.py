import pytest

from maeh.cli.app import PlanApp
from maeh.core.config import Config
from maeh.core.models import Node, PlanTree


@pytest.mark.asyncio
async def test_app_mounts_tree(tmp_path):
    tree = PlanTree(Node("r", "root", children=[Node("c", "child")]))
    app = PlanApp(tree, Config(maeh_home=tmp_path))
    async with app.run_test() as pilot:
        await pilot.pause()
        from maeh.cli.widgets.plan_tree import PlanTreeWidget

        assert app.query_one(PlanTreeWidget) is not None
