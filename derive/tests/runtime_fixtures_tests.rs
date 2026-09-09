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
    assert_linked_fixture_succeeds("cross_crate");
    assert_linked_fixture_succeeds("duplicate_id");
    assert_linked_fixture_succeeds("missing_target");
    assert_missing_runtime_fixture_fails();
    assert_missing_runtime_fixture_preserves_validation_error();
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

/// Runs one linked-workspace binary that must complete successfully.
fn assert_linked_fixture_succeeds(binary: &str) {
    let output = run_linked_fixture(binary);
    assert!(
        output.status.success(),
        "linked-workspace {binary} fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks the missing-runtime diagnostic without relying on the test crate's
/// development dependencies.
fn assert_missing_runtime_fixture_fails() {
    let output = run_fixture("missing");
    assert!(!output.status.success(), "missing runtime fixture compiled");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Model derive requires the `qubit-model-metadata` dependency"),
        "unexpected missing runtime diagnostic: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks that a missing runtime dependency preserves independent validation
/// diagnostics from the same model declaration.
fn assert_missing_runtime_fixture_preserves_validation_error() {
    let output = run_fixture("missing-invalid");
    assert!(!output.status.success(), "missing-invalid runtime fixture compiled");
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains("Model derive requires the `qubit-model-metadata` dependency"),
        "missing runtime diagnostic: {diagnostic}"
    );
    assert!(
        diagnostic.contains("Model does not support tuple structs"),
        "missing validation diagnostic: {diagnostic}"
    );
}

/// Runs `cargo check` for one standalone fixture with an isolated target dir.
fn run_fixture(name: &str) -> std::process::Output {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/runtime-fixtures").join(name);
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "qubit-model-derive-runtime-fixture-{}-{}",
                name,
                std::process::id()
            ))
        });
    Command::new(env!("CARGO"))
        .arg("check")
        .arg("--offline")
        .arg("--quiet")
        .current_dir(fixture_dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("runtime fixture cargo check should start")
}

/// Runs one collector binary from the cross-crate registration fixture.
fn run_linked_fixture(binary: &str) -> std::process::Output {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/runtime-fixtures/linked-workspace");
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("qubit-model-derive-linked-fixture-{}", std::process::id()))
        });
    Command::new(env!("CARGO"))
        .args(["run", "--offline", "--quiet", "-p", "collector", "--bin", binary])
        .args((binary != "cross_crate").then_some("--features"))
        .args((binary == "duplicate_id").then_some("duplicate-fixture"))
        .args((binary == "missing_target").then_some("missing-fixture"))
        .current_dir(fixture_dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("linked runtime fixture cargo run should start")
}
