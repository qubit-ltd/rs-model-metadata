// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Independent consumer builds for actual accessor feature combinations.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

#[path = "support/temporary_target_dir.rs"]
mod temporary_target_dir;

use temporary_target_dir::TemporaryTargetDir;

/// Checks all declared presence axes and the intersecting failure cases.
#[test]
fn test_cfg_accessor_feature_matrix() {
    let scratch = TemporaryTargetDir::new("qubit-model-cfg-fixture");
    let fixture = prepare_cfg_fixture(scratch.path());
    let fetched = Command::new(env!("CARGO"))
        .args(["fetch", "--locked", "--manifest-path"])
        .arg(&fixture)
        .output()
        .expect("cfg fixture dependency fetch should start");
    assert!(
        fetched.status.success(),
        "failed to fetch cfg fixture dependencies: {}",
        String::from_utf8_lossy(&fetched.stderr)
    );
    let target_dir = TemporaryTargetDir::new("qubit-model-cfg");
    let target = target_dir.path().join("cfg-accessors-runtime");
    let axes = ["accessors", "alternate", "impl-enabled"];
    for mask in 0..8 {
        let features: Vec<_> = axes
            .iter()
            .enumerate()
            .filter_map(|(index, feature)| (mask & (1 << index) != 0).then_some(*feature))
            .collect();
        for action in ["check", "test"] {
            let output = run_fixture(&fixture, &target, action, &features.join(","));
            assert!(
                output.status.success(),
                "cfg fixture {action} features {features:?}:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
    for (features, diagnostic) in [
        ("duplicate-getter", "duplicate property getter `paired`"),
        ("duplicate-setter", "duplicate property setter `paired`"),
        ("mismatch", "PropertyOutputCompatible<bool>"),
        ("alternate,mismatch", "PropertyOutputCompatible<bool>"),
    ] {
        let output = run_fixture(&fixture, &target, "check", features);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "invalid cfg features {features} compiled");
        assert!(
            stderr.contains(diagnostic),
            "cfg features {features} missing {diagnostic}: {stderr}"
        );
        let normalized_stderr = stderr.replace('\\', "/");
        assert!(
            normalized_stderr.contains("src/lib.rs:"),
            "cfg diagnostic needs source position: {stderr}"
        );
    }
}

/// Copies the standalone fixture and points its reflect dependency at the
/// explicitly selected integration checkout when the test runs in a worktree.
fn prepare_cfg_fixture(directory: &Path) -> PathBuf {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = package_root.join("tests/runtime-fixtures/cfg-accessors");
    let destination = directory.join("cfg-accessors");
    fs::create_dir_all(destination.join("src")).expect("fixture sources");
    fs::copy(source.join("src/lib.rs"), destination.join("src/lib.rs")).expect("fixture library");
    fs::copy(source.join("Cargo.lock"), destination.join("Cargo.lock")).expect("fixture lockfile");

    let reflect_root = std::env::var_os("QUBIT_REFLECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| package_root.join("../../rs-reflect"));
    let manifest = fs::read_to_string(source.join("Cargo.toml")).expect("fixture manifest");
    let manifest = manifest
        .replace(
            "qubit-model-derive = { path = \"../../../\" }",
            &format!("qubit-model-derive = {{ path = \"{}\" }}", package_root.display()),
        )
        .replace(
            "model_runtime = { package = \"qubit-model-metadata\", path = \"../../../..\", features = [\"generic\"] }",
            &format!(
                "model_runtime = {{ package = \"qubit-model-metadata\", path = \"{}\", features = [\"generic\"] }}",
                package_root.join("..").display()
            ),
        )
        .replace(
            "qubit-reflect = { path = \"../../../../../rs-reflect\" }",
            &format!("qubit-reflect = {{ path = \"{}\" }}", reflect_root.display()),
        );
    fs::write(destination.join("Cargo.toml"), manifest).expect("fixture manifest copy");
    destination.join("Cargo.toml")
}

/// Runs one standalone consumer command, using a child target directory to
/// avoid Cargo's parent build lock. Returns both output streams and status;
/// panics only if the Cargo process cannot be started.
fn run_fixture(manifest: &Path, target: &Path, action: &str, features: &str) -> Output {
    let mut command = Command::new(env!("CARGO"));
    command
        .args([action, "--offline", "--locked", "--manifest-path"])
        .arg(manifest)
        .env("CARGO_TARGET_DIR", target);
    if !features.is_empty() {
        command.args(["--features", features]);
    }
    command.output().expect("cfg fixture Cargo process should start")
}
