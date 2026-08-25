#!/usr/bin/env python3
"""Migrate #[field(...)] to standalone field helper attributes."""

from __future__ import annotations

import sys
from pathlib import Path


def split_top_level_commas(content: str) -> list[str]:
    parts: list[str] = []
    current: list[str] = []
    depth = 0
    for char in content:
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            part = "".join(current).strip()
            if part:
                parts.append(part)
            current = []
            continue
        current.append(char)
    tail = "".join(current).strip()
    if tail:
        parts.append(tail)
    return parts


def convert_field_item(item: str) -> list[str]:
    item = item.strip()
    if not item:
        return []
    if item == "index":
        return ["indexed"]
    if item.startswith("index("):
        raise ValueError(f"unsupported field attribute item: {item}")
    return [item]


def field_inner_to_attributes(inner: str) -> list[str]:
    return [converted for item in split_top_level_commas(inner) for converted in convert_field_item(item)]


def find_field_attribute_spans(text: str) -> list[tuple[int, int, str]]:
    spans: list[tuple[int, int, str]] = []
    marker = "#[field("
    index = 0
    while True:
        start = text.find(marker, index)
        if start == -1:
            break
        inner_start = start + len(marker)
        depth = 1
        cursor = inner_start
        while cursor < len(text) and depth > 0:
            char = text[cursor]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            cursor += 1
        if depth != 0:
            raise ValueError(f"unbalanced parentheses in {text[start:start + 40]!r}...")
        if cursor < len(text) and text[cursor] == "]":
            cursor += 1
        inner = text[inner_start : cursor - 1]
        spans.append((start, cursor, inner))
        index = cursor
    return spans


def migrate_text(text: str) -> str:
    spans = find_field_attribute_spans(text)
    if not spans:
        return text
    for start, end, inner in reversed(spans):
        attributes = field_inner_to_attributes(inner)
        replacement = "\n    ".join(f"#[{attribute}]" for attribute in attributes)
        text = text[:start] + replacement + text[end:]
    return text


def migrate_file(path: Path) -> bool:
    original = path.read_text()
    migrated = migrate_text(original)
    if migrated == original:
        return False
    path.write_text(migrated)
    return True


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print("usage: migrate_field_attributes.py <path>...", file=sys.stderr)
        return 1
    changed = 0
    for argument in argv[1:]:
        path = Path(argument)
        if path.is_dir():
            files = sorted(path.rglob("*.rs"))
        else:
            files = [path]
        for file_path in files:
            if migrate_file(file_path):
                print(file_path)
                changed += 1
    print(f"migrated {changed} file(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
