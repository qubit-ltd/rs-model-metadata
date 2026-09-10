#!/bin/bash
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

# Checks project-owned CI entry points and active documentation links.
# Executable Markdown examples run in the regular Cargo test matrix through
# tests/documentation_examples_tests.rs; do not compile them a second time here.
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
python3 "$PROJECT_ROOT/tests/ci_wrapper_tests.py"
python3 "$PROJECT_ROOT/tests/doc_link_tests.py"
python3 "$PROJECT_ROOT/scripts/check_doc_links.py"
