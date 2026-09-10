#!/bin/bash
# Copyright (c) 2025 - 2026 Haixing Hu.
# SPDX-License-Identifier: Apache-2.0

# Sourced only by CI/coverage entry points, never ordinary Cargo builds.
# With Rust 1.94, unused inline mappings from one integration binary can mask
# executed mappings from another binary in a combined LLVM coverage report.
# Retaining code restores those mappings without exempting uncalled functions.
# See doc/coverage_measurement.md for the reproduction and negative control.
set -euo pipefail

if [[ ${CARGO_ENCODED_RUSTFLAGS+x} ]]; then
    # Cargo gives encoded flags precedence even when explicitly empty.
    export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+${CARGO_ENCODED_RUSTFLAGS}$'\x1f'}-C"$'\x1f'"link-dead-code=yes"
else
    export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-C link-dead-code=yes"
fi
