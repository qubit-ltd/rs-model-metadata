// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Independent consumer builds for actual accessor feature combinations.

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
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/runtime-fixtures/cfg-accessors/Cargo.toml");
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
                "cfg fixture {action} features {features:?}: {}",
                String::from_utf8_lossy(&output.stderr)
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
