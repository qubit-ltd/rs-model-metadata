#!/usr/bin/env python3
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

"""Check active local Markdown destinations and heading anchors without network IO.

Fenced/inline code and comments are ignored. Inline/image/reference links, ATX
and single-line Setext headings, and explicit HTML anchors are supported.
Linked Markdown is followed within the project, except historical archive
directories. Legacy source :line annotations check the file, not old line bounds.
"""

import argparse
import html
from pathlib import Path
import re
import sys
import unicodedata
from urllib.parse import unquote, urlsplit


DEFAULT_DOCUMENTS = (
    "README.md", "README.zh_CN.md", "derive/README.md", "derive/README.zh_CN.md",
    "doc/user_guide.md", "doc/user_guide.zh_CN.md",
    "derive/doc/user_guide.md", "derive/doc/user_guide.zh_CN.md",
    "derive/doc/rs-model-derive-final-design.md",
    "derive/doc/rs-model-derive-final-design.zh_CN.md",
    "doc/coverage_measurement.md", "doc/coverage_measurement.zh_CN.md",
)


def blank(text):
    """Hide syntax while retaining offsets and source line numbers."""
    return re.sub(r"[^\n]", " ", text)


def prose(text):
    """Remove fenced/indented code and HTML comments before link inspection."""
    text = re.sub(r"<!--.*?-->", lambda match: blank(match[0]), text, flags=re.S)
    result = []
    fence = None
    for line in text.splitlines(keepends=True):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line.rstrip("\n"))
        if fence:
            result.append(blank(line))
            if marker and marker[1][0] == fence[0] and len(marker[1]) >= fence[1] and not marker[2].strip():
                fence = None
        elif marker:
            fence = (marker[1][0], len(marker[1]))
            result.append(blank(line))
        else:
            result.append(blank(line) if line.startswith(("    ", "\t")) else line)
    return "".join(result)


def anchors(text):
    """Generate heading IDs, including duplicate-heading suffixes."""
    found = set(re.findall(r'<a\b[^>]*\b(?:id|name)=["\']([^"\']+)', text, flags=re.I))
    lines = text.splitlines()
    for index, line in enumerate(lines):
        heading = re.match(r"^ {0,3}#{1,6}[ \t]+(.+?)(?:[ \t]+#+[ \t]*)?$", line)
        if heading:
            title = heading[1]
        elif index and re.fullmatch(r" {0,3}(?:=+|-+)\s*", line) and lines[index - 1].strip():
            title = lines[index - 1].strip()
        else:
            continue
        title = html.unescape(re.sub(r"<[^>]*>", "", title)).lower()
        title = re.sub(r"\[([^]]+)\]\([^)]*\)", r"\1", title)
        slug = "".join(character for character in title if character in "-_" or character.isspace()
                       or unicodedata.category(character)[0] in "LNM")
        slug = re.sub(r"\s", "-", slug)
        candidate, suffix = slug, 0
        while candidate in found:
            suffix += 1
            candidate = f"{slug}-{suffix}"
        found.add(candidate)
    return found


def closing(text, start, opener, closer):
    """Find a balanced delimiter, preserving escaped destination punctuation."""
    depth = 1
    index = start + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
            continue
        if text[index] == opener:
            depth += 1
        elif text[index] == closer:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def destination(body):
    """Read an angle-bracket or bare destination before an optional title."""
    body = body.strip()
    if body.startswith("<"):
        value = body[1:body.find(">")]
    else:
        value = body.split(maxsplit=1)[0] if body else ""
    return html.unescape(re.sub(r"\\([\\()\[\]<> ])", r"\1", value))


def reference_id(label):
    return " ".join(label.split()).casefold()


def links(text, references):
    """Yield destination offsets, including images nested inside link labels."""
    index = 0
    while index < len(text):
        if text[index] == "\\":
            index += 2
            continue
        if text[index] != "[":
            index += 1
            continue
        end = closing(text, index, "[", "]")
        if end is None:
            break
        label = text[index + 1:end]
        for offset, target in links(label, references):
            yield index + 1 + offset, target
        after = end + 1
        if after < len(text) and text[after] == "(":
            finish = closing(text, after, "(", ")")
            if finish is not None:
                yield index, destination(text[after + 1:finish])
                index = finish + 1
                continue
        elif after < len(text) and text[after] == "[":
            finish = closing(text, after, "[", "]")
            if finish is not None:
                identifier = reference_id(text[after + 1:finish] or label)
                if identifier in references:
                    yield index, references[identifier]
                index = finish + 1
                continue
        elif reference_id(label) in references:
            yield index, references[reference_id(label)]
        index = end + 1


def document_links(text):
    references = {}
    pattern = r"(?m)^ {0,3}\[([^]\n]+)\]:[ \t]*(.+)$"
    for match in re.finditer(pattern, text):
        references[reference_id(match[1])] = destination(match[2])
    text = re.sub(pattern, lambda match: blank(match[0]), text)
    text = re.sub(r"(`+)(.*?)\1", lambda match: blank(match[0]), text, flags=re.S)
    return links(text, references)


def check(root, names):
    pending = [(root / name).resolve() for name in names]
    visited = set()
    failures = []
    local_count = 0
    while pending:
        source = pending.pop()
        if source in visited:
            continue
        visited.add(source)
        try:
            text = prose(source.read_text(encoding="utf-8"))
        except (OSError, UnicodeError) as error:
            failures.append(f"{source}: cannot read document: {error}")
            continue
        for offset, target in document_links(text):
            # Source-coordinate links predate GitHub-style #L fragments.
            target = re.sub(r":\d+$", "", target)
            url = urlsplit(target)
            if url.scheme or url.netloc:
                continue
            local_count += 1
            location = source.parent / unquote(url.path) if url.path else source
            location = location.resolve()
            line = text.count("\n", 0, offset) + 1
            context = f"{source}:{line}: {target!r}"
            if not location.exists():
                failures.append(f"{context}: missing destination")
                continue
            if location.suffix.lower() != ".md":
                continue
            if url.fragment:
                try:
                    target_text = prose(location.read_text(encoding="utf-8"))
                    if unquote(url.fragment) not in anchors(target_text):
                        failures.append(f"{context}: missing anchor")
                except (OSError, UnicodeError) as error:
                    failures.append(f"{context}: cannot read destination: {error}")
            if location.is_relative_to(root) and "archive" not in location.relative_to(root).parts:
                pending.append(location)
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f"Checked {len(visited)} active documents and {local_count} local links.")
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("documents", nargs="*")
    args = parser.parse_args()
    return check(args.root.resolve(), args.documents or DEFAULT_DOCUMENTS)


if __name__ == "__main__":
    sys.exit(main())
