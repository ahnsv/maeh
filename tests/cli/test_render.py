from maeh.cli.render import OutputFormat, render


def test_json():
    assert render({"a": 1}, OutputFormat.json) == '{\n  "a": 1\n}'


def test_plaintext_flattens():
    assert render({"a": {"b": 1}}, OutputFormat.plaintext) == "a.b = 1"


def test_yaml():
    assert render({"a": 1}, OutputFormat.yaml) == "a: 1"
