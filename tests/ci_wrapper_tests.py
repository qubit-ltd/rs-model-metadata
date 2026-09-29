#!/usr/bin/env python3
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

"""Project CI wrappers preserve caller flags and coverage mappings."""

import json
import os
import re
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

    def test_downstream_repositories_use_immutable_revisions(self):
        workflow = (PROJECT_ROOT / ".github/workflows/rs-platform-downstream.yml").read_text(
            encoding="utf-8"
        )
        checkouts = re.findall(
            r"repository:\s*(qubit-ltd/[^\s]+)\s*\n\s*ref:\s*([^\s]+)",
            workflow,
        )
        expected_repositories = {
            "qubit-ltd/rs-reflect",
            "qubit-ltd/rs-platform",
            "qubit-ltd/rs-id",
            "qubit-ltd/rs-datatype",
            "qubit-ltd/rs-redact",
            "qubit-ltd/rs-redact-derive",
            "qubit-ltd/rs-codec",
            "qubit-ltd/rs-validator",
            "qubit-ltd/rs-validation-rules",
        }
        repositories = {repository for repository, _ in checkouts}
        self.assertEqual(repositories, expected_repositories)
        for repository, revision in checkouts:
            with self.subTest(repository=repository):
                self.assertRegex(revision, r"^[0-9a-f]{40}$")

    def test_downstream_cross_repository_checkouts_declare_dependency_token(self):
        workflow = (PROJECT_ROOT / ".github/workflows/rs-platform-downstream.yml").read_text(
            encoding="utf-8"
        )
        blocks = re.findall(
            r"^      - (?:name: [^\n]+\n        )?uses: actions/checkout@v6\n"
            r"        with:\n(?P<with>(?:          .*\n)+)",
            workflow,
            re.MULTILINE,
        )
        dependency_checkouts = [
            block for block in blocks if re.search(r"^\s+repository:\s*qubit-ltd/", block)
        ]

        self.assertEqual(len(dependency_checkouts), 9)
        for checkout in dependency_checkouts:
            repository = re.search(r"^\s+repository:\s*([^\s]+)", checkout, re.MULTILINE)
            with self.subTest(repository=repository.group(1) if repository else "missing"):
                self.assertIsNotNone(repository)
                self.assertIn("token: ${{ secrets.DEPENDENCY_TOKEN || github.token }}", checkout)

    def test_cross_platform_cargo_commands_cover_locked_workspace(self):
        workflow = (PROJECT_ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        jobs = workflow.split("\n  cross-platform-tests:\n", maxsplit=1)
        self.assertEqual(len(jobs), 2, "cross-platform-tests job is missing")
        job = re.split(r"\n  [a-zA-Z0-9_-]+:\n", jobs[1], maxsplit=1)[0]
        commands = re.findall(r"^\s+- run:\s*(cargo\s+(?:test|clippy)\b.*)$", job, re.MULTILINE)

        self.assertEqual(len(commands), 2, "expected cross-platform test and clippy commands")
        by_command = {"test": None, "clippy": None}
        for command in commands:
            if command.startswith("cargo test "):
                by_command["test"] = command
            elif command.startswith("cargo clippy "):
                by_command["clippy"] = command

        for name, command in by_command.items():
            with self.subTest(command=name):
                self.assertIsNotNone(command, f"cross-platform {name} command is missing")
                if command is None:
                    continue
                self.assertIn("--workspace", command)
                self.assertIn("--locked", command)
                self.assertIn("--all-features", command)

    def test_cross_platform_toolchain_installs_clippy_component(self):
        workflow = (PROJECT_ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        jobs = workflow.split("\n  cross-platform-tests:\n", maxsplit=1)
        self.assertEqual(len(jobs), 2, "cross-platform-tests job is missing")
        job = re.split(r"\n  [a-zA-Z0-9_-]+:\n", jobs[1], maxsplit=1)[0]
        setup = re.search(
            r"- uses: actions-rust-lang/setup-rust-toolchain@v1\n(?P<with>\s+with:\n(?:\s{10,}.*\n)+)",
            job,
        )

        self.assertIsNotNone(setup, "cross-platform Rust toolchain setup is missing")
        self.assertRegex(setup.group("with"), r"(?m)^\s+components:\s*clippy\s*$")


if __name__ == "__main__":
    unittest.main()
