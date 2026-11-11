#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)

worker_package="$project_root/worker/package.json"
if [[ -f "$worker_package" ]] && grep -Eq '"build:test-worker"[[:space:]]*:' "$worker_package"; then
    command -v node >/dev/null || { echo "error: Node.js is required to build the test worker" >&2; exit 1; }
    command -v corepack >/dev/null || { echo "error: Corepack is required to build the test worker" >&2; exit 1; }
    (
        cd "$project_root/worker"
        corepack pnpm install --frozen-lockfile --ignore-scripts
        corepack pnpm run build:test-worker
    )
fi

exec "${RS_INFRA_PYTHON:-python3}" "$project_root/scripts/ci-check-locked.py" "$@"
