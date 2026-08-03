import ast
import pathlib

CORE = pathlib.Path(__file__).resolve().parent.parent / "src" / "maeh" / "core"


def _imports(path):
    tree = ast.parse(path.read_text())
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module:
            yield node.module
        if isinstance(node, ast.Import):
            for n in node.names:
                yield n.name


def test_core_never_imports_cli_or_textual():
    offenders = []
    for py in CORE.rglob("*.py"):
        for mod in _imports(py):
            if mod.startswith(("maeh.cli", "textual")):
                offenders.append((str(py), mod))
    assert offenders == [], offenders
