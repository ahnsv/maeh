from __future__ import annotations

import json
from enum import StrEnum


class OutputFormat(StrEnum):
    json = "json"
    yaml = "yaml"
    plaintext = "plaintext"


def _flatten(data, prefix: str = "") -> list[str]:
    out: list[str] = []
    if isinstance(data, dict):
        for k, v in data.items():
            out += _flatten(v, f"{prefix}{k}.")
    elif isinstance(data, list):
        for i, v in enumerate(data):
            out += _flatten(v, f"{prefix}{i}.")
    else:
        out.append(f"{prefix.rstrip('.')} = {data}")
    return out


def render(data, fmt: OutputFormat) -> str:
    if fmt is OutputFormat.json:
        return json.dumps(data, ensure_ascii=False, indent=2)
    if fmt is OutputFormat.yaml:
        import yaml

        return yaml.safe_dump(data, allow_unicode=True, sort_keys=False).rstrip()
    return "\n".join(_flatten(data))
