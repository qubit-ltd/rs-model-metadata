// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Isolated Cargo fixtures for runtime dependency resolution.

use std::path::Path;
use std::process::Command;

#[path = "support/temporary_target_dir.rs"]
mod temporary_target_dir;

use temporary_target_dir::TemporaryTargetDir;

/// Checks normal, renamed, and absent runtime dependency declarations.
#[test]
fn test_runtime_dependency_fixtures() {
    let target_dir = TemporaryTargetDir::new("qubit-model-derive-runtime-fixtures");
    assert_fixture_succeeds(target_dir.path(), "normal");
    assert_fixture_succeeds(target_dir.path(), "renamed");
    assert_linked_fixture_succeeds(target_dir.path(), "cross_crate");
    assert_linked_fixture_succeeds(target_dir.path(), "duplicate_id");
    assert_linked_fixture_succeeds(target_dir.path(), "missing_target");
    assert_missing_runtime_fixture_fails(target_dir.path());
    assert_missing_runtime_fixture_preserves_validation_error(target_dir.path());
}

/// Runs one fixture that must compile successfully.
fn assert_fixture_succeeds(target_dir: &Path, name: &str) {
    let output = run_fixture(target_dir, name);
    assert!(
        output.status.success(),
        "{name} runtime fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Runs one linked-workspace binary that must complete successfully.
fn assert_linked_fixture_succeeds(target_dir: &Path, binary: &str) {
    let output = run_linked_fixture(target_dir, binary);
    assert!(
        output.status.success(),
        "linked-workspace {binary} fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks the missing-runtime diagnostic without relying on the test crate's
/// development dependencies.
fn assert_missing_runtime_fixture_fails(target_dir: &Path) {
    let output = run_fixture(target_dir, "missing");
    assert!(!output.status.success(), "missing runtime fixture compiled");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Model derive requires the `qubit-model-metadata` dependency"),
        "unexpected missing runtime diagnostic: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks that a missing runtime dependency preserves independent validation
/// diagnostics from the same model declaration.
fn assert_missing_runtime_fixture_preserves_validation_error(target_dir: &Path) {
    let output = run_fixture(target_dir, "missing-invalid");
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
fn run_fixture(target_dir: &Path, name: &str) -> std::process::Output {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/runtime-fixtures").join(name);
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
fn run_linked_fixture(target_dir: &Path, binary: &str) -> std::process::Output {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/runtime-fixtures/linked-workspace");
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
