#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
exec env \
    RS_CI_PROJECT_ROOT="$PROJECT_ROOT" \
    STYLE_ENFORCE_AGGREGATION_FILES=0 \
    STYLE_ENFORCE_EXPLICIT_IMPORTS=0 \
    STYLE_EXTRA_EXCLUDE_REGEX='^tests/(ui|ui_roles|runtime-fixtures)/' \
    STYLE_TEST_SUPPORT_DIR_REGEX='(^|/)(support|common|fixtures|coverage_support|ui|ui_roles|runtime-fixtures)(/|$)' \
    "$PROJECT_ROOT/.rs-ci/ci-check.sh" "$@"
