#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# The vendor CI runner invokes its own coverage script, so the parent must
# supply the same code-generation policy as the standalone coverage entry.
# shellcheck source=scripts/coverage-rustflags.sh
source "$PROJECT_ROOT/scripts/coverage-rustflags.sh"
exec env RS_CI_PROJECT_ROOT="$PROJECT_ROOT" "$PROJECT_ROOT/.rs-ci/ci-check.sh" "$@"
