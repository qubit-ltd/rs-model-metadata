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

    def test_coverage_report_expands_workspace_scope_to_package_arguments(self):
        if not os.environ.get("RS_INFRA_SHARED_ROOT"):
            self.skipTest("requires the shared rs-infra-tools checkout")
        with tempfile.TemporaryDirectory(prefix="metadata coverage report ") as directory:
            root = Path(directory)
            tools = root / ".infra" / "tools"
            tools.mkdir(parents=True)
            config = root / ".infra" / "coverage"
            config.mkdir()
            shared_lib = root / ".infra" / "lib"
            shared_lib.mkdir()
            (shared_lib / "cleanup-build-artifacts.sh").write_text("", encoding="utf-8")
            (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            (config / "coverage.json").write_text(
                json.dumps({"scope": "workspace", "exclude_packages": ["excluded"]}),
                encoding="utf-8",
            )
            metadata = {
                "workspace_root": str(root),
                "workspace_default_members": ["pkg-root"],
                "workspace_members": ["pkg-root", "pkg-member", "pkg-excluded"],
                "packages": [
                    {"id": "pkg-root", "name": "root-package", "manifest_path": str(root / "Cargo.toml")},
                    {"id": "pkg-member", "name": "member-package", "manifest_path": str(root / "member/Cargo.toml")},
                    {"id": "pkg-excluded", "name": "excluded", "manifest_path": str(root / "excluded/Cargo.toml")},
                ],
            }
            metadata_path = root / "metadata.json"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            shared_root = Path(os.environ["RS_INFRA_SHARED_ROOT"])
            shutil.copyfile(
                shared_root / ".infra/lib/coverage-report.sh",
                tools / "coverage-report.sh",
            )

            cargo_bin = root / "bin"
            cargo_bin.mkdir()
            records = root / "cargo-arguments.jsonl"
            step_summary = root / "step-summary.md"
            fake_cargo = cargo_bin / "cargo"
            fake_cargo.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, pathlib, sys\n"
                "args = sys.argv[1:]\n"
                "if args[0] == 'metadata':\n"
                "    print(pathlib.Path(os.environ['CARGO_METADATA_FIXTURE']).read_text())\n"
                "    raise SystemExit(0)\n"
                "with open(os.environ['CARGO_ARGUMENTS'], 'a', encoding='utf-8') as output:\n"
                "    output.write(json.dumps(args) + '\\n')\n"
                "if '--output-path' in args:\n"
                "    target = pathlib.Path(args[args.index('--output-path') + 1])\n"
                "    content = json.dumps({'data':[{'totals':{'functions':{'covered':1,'count':1},'lines':{'covered':1,'count':1},'regions':{'covered':1,'count':1}}}]}) if '--json' in args else 'coverage report\\n'\n"
                "    target.write_text(content, encoding='utf-8')\n"
                "if '--output-dir' in args:\n"
                "    pathlib.Path(args[args.index('--output-dir') + 1]).mkdir(parents=True, exist_ok=True)\n",
                encoding="utf-8",
            )
            fake_cargo.chmod(0o755)
            environment = os.environ.copy()
            environment.update(
                {
                    "PATH": f"{cargo_bin}{os.pathsep}{environment['PATH']}",
                    "CARGO_METADATA_FIXTURE": str(metadata_path),
                    "CARGO_ARGUMENTS": str(records),
                    "GITHUB_STEP_SUMMARY": str(step_summary),
                }
            )
            result = subprocess.run(
                ["bash", str(tools / "coverage-report.sh")],
                cwd=root,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            summary = step_summary.read_text(encoding="utf-8")
            self.assertIn("Full text report", summary)
            self.assertNotIn("coverage report", summary)
            self.assertLess(len(summary.encode("utf-8")), 1024 * 1024)
            report_calls = [json.loads(line) for line in records.read_text().splitlines()]
            self.assertEqual(len(report_calls), 5)
            for arguments in report_calls:
                with self.subTest(arguments=arguments):
                    self.assertNotIn("--workspace", arguments)
                    self.assertNotIn("--exclude", arguments)
                    self.assertEqual(
                        [arguments[index + 1] for index, value in enumerate(arguments[:-1]) if value == "--package"],
                        ["root-package", "member-package"],
                    )


if __name__ == "__main__":
    unittest.main()
