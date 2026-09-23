#!/usr/bin/env python3
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

"""Project CI wrappers preserve caller flags and coverage mappings."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


PROJECT_ROOT = Path(__file__).resolve().parents[1]
WRAPPERS = ("ci-check.sh", "coverage.sh")


class CiWrapperTests(unittest.TestCase):
    """Exercise the rs-infra wrappers against a recording tool runner."""

    def run_wrapper(self, wrapper, flags, arguments=(), exit_code=0):
        with tempfile.TemporaryDirectory(prefix="model CI wrappers ") as directory:
            root = Path(directory)
            tools = root / ".infra" / "tools"
            tools.mkdir(parents=True)
            (root / "scripts").mkdir()
            shutil.copyfile(PROJECT_ROOT / wrapper, root / wrapper)
            shutil.copyfile(
                PROJECT_ROOT / "scripts" / "coverage-rustflags.sh",
                root / "scripts" / "coverage-rustflags.sh",
            )
            (tools / "cleanup-build-artifacts.sh").write_text("#!/usr/bin/env bash\n")
            prepare = tools / "prepare-local-path-dependencies.sh"
            prepare.write_text("#!/usr/bin/env bash\n")
            prepare.chmod(0o755)
            runner = tools / "infra-tool.sh"
            runner.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, sys\n"
                "print(json.dumps({\n"
                "  'arguments': sys.argv[1:],\n"
                "  'rustflags': os.environ.get('RUSTFLAGS'),\n"
                "  'encoded': os.environ.get('CARGO_ENCODED_RUSTFLAGS'),\n"
                "}))\n"
                "sys.exit(int(os.environ['FIXTURE_EXIT_CODE']))\n",
                encoding="utf-8",
            )
            runner.chmod(0o755)
            report = tools / "coverage-report.sh"
            report.write_text("#!/usr/bin/env bash\necho report-complete\n")
            report.chmod(0o755)

            environment = os.environ.copy()
            for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
                environment.pop(name, None)
            environment.update(flags)
            environment["FIXTURE_EXIT_CODE"] = str(exit_code)
            result = subprocess.run(
                ["bash", str(root / wrapper), *arguments],
                cwd=PROJECT_ROOT.parent,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, exit_code, result.stderr)
            data = json.loads(result.stdout.splitlines()[0])
            expected_task = "rs-infra-ci" if wrapper == "ci-check.sh" else "rs-infra-coverage"
            expected_args = [expected_task, "--project", str(root)]
            expected_args += ([*arguments, "check"] if wrapper == "ci-check.sh" else ["collect", *arguments])
            self.assertEqual(data["arguments"], expected_args)
            return data

    def test_default_flags_retain_coverage_mappings(self):
        for wrapper in WRAPPERS:
            with self.subTest(wrapper=wrapper):
                result = self.run_wrapper(wrapper, {})
                self.assertEqual(result["rustflags"], "-C link-dead-code=yes")
                self.assertIsNone(result["encoded"])

    def test_plain_flags_are_preserved(self):
        for wrapper in WRAPPERS:
            with self.subTest(wrapper=wrapper):
                result = self.run_wrapper(wrapper, {"RUSTFLAGS": "-D warnings"})
                self.assertEqual(result["rustflags"], "-D warnings -C link-dead-code=yes")

    def test_encoded_flags_keep_argument_boundaries_and_precedence(self):
        encoded = "--remap-path-prefix\x1f/source tree=/mapped tree"
        for wrapper in WRAPPERS:
            with self.subTest(wrapper=wrapper):
                result = self.run_wrapper(
                    wrapper,
                    {"RUSTFLAGS": "ignored by Cargo", "CARGO_ENCODED_RUSTFLAGS": encoded},
                )
                self.assertEqual(result["encoded"], encoded + "\x1f-C\x1flink-dead-code=yes")
                self.assertEqual(result["rustflags"], "ignored by Cargo")

    def test_explicit_empty_encoded_flags_remain_authoritative(self):
        for wrapper in WRAPPERS:
            with self.subTest(wrapper=wrapper):
                result = self.run_wrapper(
                    wrapper,
                    {"RUSTFLAGS": "ignored by Cargo", "CARGO_ENCODED_RUSTFLAGS": ""},
                )
                self.assertEqual(result["encoded"], "-C\x1flink-dead-code=yes")

    def test_arguments_and_tool_failure_are_forwarded(self):
        for wrapper in WRAPPERS:
            with self.subTest(wrapper=wrapper):
                self.run_wrapper(wrapper, {}, ("json", "path with spaces", "$(literal)"), exit_code=23)


if __name__ == "__main__":
    unittest.main()
