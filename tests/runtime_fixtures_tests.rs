// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Isolated Cargo fixtures for runtime dependency resolution.

use std::process::Command;

/// Checks normal, renamed, and absent runtime dependency declarations.
#[test]
fn test_runtime_dependency_fixtures() {
    assert_fixture_succeeds("normal");
    assert_fixture_succeeds("renamed");
    assert_missing_runtime_fixture_fails();
}

/// Runs one fixture that must compile successfully.
fn assert_fixture_succeeds(name: &str) {
    let output = run_fixture(name);
    assert!(
        output.status.success(),
        "{name} runtime fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks the missing-runtime diagnostic without relying on the test crate's
/// development dependencies.
fn assert_missing_runtime_fixture_fails() {
    let output = run_fixture("missing");
    assert!(!output.status.success(), "missing runtime fixture compiled");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "Model derive requires the `qubit-model-metadata` dependency"
        ),
        "unexpected missing runtime diagnostic: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Runs `cargo check` for one standalone fixture with an isolated target dir.
fn run_fixture(name: &str) -> std::process::Output {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/runtime-fixtures").join(name);
    let target_dir = std::env::temp_dir().join(format!(
        "qubit-model-derive-runtime-fixture-{}-{}",
        name,
        std::process::id()
    ));
    Command::new(env!("CARGO"))
        .arg("check")
        .arg("--quiet")
        .current_dir(fixture_dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("runtime fixture cargo check should start")
}
