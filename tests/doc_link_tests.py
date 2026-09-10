#!/usr/bin/env python3
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

"""Exercise the documentation checker through its command-line contract."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


CHECKER = Path(__file__).resolve().parents[1] / "scripts" / "check_doc_links.py"


class DocumentationLinkTests(unittest.TestCase):
    def check(self, files):
        with tempfile.TemporaryDirectory(prefix="model doc links ") as directory:
            root = Path(directory)
            for name, content in files.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(CHECKER), "--root", str(root), "README.md"],
                capture_output=True, text=True, check=False,
            )

    def test_unicode_duplicate_headings_and_encoded_paths(self):
        result = self.check({
            "README.md": "[guide](<doc/guide%20%28copy%29.md#字段-api>)\n"
                         "[second](doc/guide%20%28copy%29.md#字段-api-1)\n"
                         "[setext](doc/guide%20%28copy%29.md#plain-title)\n"
                         "![image](doc/icon.svg)\n[site](https://example.invalid/)\n",
            "doc/guide (copy).md": "# 字段 `API`\n# 字段 API\nPlain title\n===\n",
            "doc/icon.svg": "<svg/>\n",
        })
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("4 local links", result.stdout)

    def test_broken_destination_reports_source_line(self):
        result = self.check({"README.md": "# Intro\n[missing](doc/missing.md)\n"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("README.md:2", result.stderr)
        self.assertIn("doc/missing.md", result.stderr)

    def test_broken_fragment_is_not_accepted_as_existing_file(self):
        result = self.check({"README.md": "# Intro\n[bad](#absent)\n"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing anchor", result.stderr)

    def test_reference_links_and_recursive_documents_are_checked(self):
        result = self.check({
            "README.md": "[read][GUIDE]\n\n[guide]: doc/guide.md\n",
            "doc/guide.md": "[missing](missing.md)\n",
        })
        self.assertEqual(result.returncode, 1)
        self.assertIn("doc/guide.md:1", result.stderr)

    def test_code_examples_and_comments_do_not_create_links(self):
        result = self.check({
            "README.md": "# Intro\n`[not a link](missing.md)`\n"
                         "~~~markdown\n[example](missing.md)\n~~~\n"
                         "<!-- [comment](missing.md) -->\n[real](#intro)\n",
        })
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("1 local links", result.stdout)

    def test_archived_documents_are_existing_historical_leaves(self):
        result = self.check({
            "README.md": "[historical](doc/archive/old.md)\n[source](src/item.rs:12)\n",
            "doc/archive/old.md": "[historical path](old-checkout.rs)\n",
            "src/item.rs": "// Historical source coordinates are not Markdown anchors.\n",
        })
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_balanced_destination_parentheses_and_explicit_anchor(self):
        result = self.check({
            "README.md": '[guide](doc/guide(copy).md#custom "Guide title")\n',
            "doc/guide(copy).md": '<a id="custom"></a>\n',
        })
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
