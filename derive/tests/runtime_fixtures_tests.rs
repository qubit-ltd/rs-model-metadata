// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Isolated Cargo fixtures for runtime dependency resolution.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[path = "support/temporary_target_dir.rs"]
mod temporary_target_dir;

use temporary_target_dir::TemporaryTargetDir;

/// Checks normal, renamed, and absent runtime dependency declarations.
#[test]
fn test_runtime_dependency_fixtures() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/runtime-fixtures");
    let fixture_scratch = TemporaryTargetDir::new("qubit-model-derive-fixture-sources");
    let fixture_root = prepare_runtime_fixtures(&source_root, fixture_scratch.path());
    prefetch_runtime_fixture_dependencies(&fixture_root);
    let target_dir = TemporaryTargetDir::new("qubit-model-derive-runtime-fixtures");
    assert_fixture_succeeds(&fixture_root, target_dir.path(), "normal");
    assert_fixture_succeeds(&fixture_root, target_dir.path(), "renamed");
    assert_linked_fixture_succeeds(&fixture_root, target_dir.path(), "cross_crate");
    assert_linked_fixture_succeeds(&fixture_root, target_dir.path(), "duplicate_id");
    assert_linked_fixture_succeeds(&fixture_root, target_dir.path(), "missing_target");
    assert_missing_runtime_fixture_fails(&fixture_root, target_dir.path());
    assert_missing_runtime_fixture_preserves_validation_error(&fixture_root, target_dir.path());
}

/// Downloads fixture dependencies before the isolated builds run offline.
fn prefetch_runtime_fixture_dependencies(fixture_root: &Path) {
    for fixture in ["normal", "renamed", "missing", "missing-invalid", "linked-workspace"] {
        let manifest = fixture_root.join(fixture).join("Cargo.toml");
        let output = Command::new(env!("CARGO"))
            .args(["fetch", "--manifest-path"])
            .arg(&manifest)
            .output()
            .expect("runtime fixture dependency fetch should start");
        assert!(
            output.status.success(),
            "failed to fetch runtime fixture dependencies for {}: {}",
            manifest.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Runs one fixture that must compile successfully.
fn assert_fixture_succeeds(fixture_root: &Path, target_dir: &Path, name: &str) {
    let output = run_fixture(fixture_root, target_dir, name);
    assert!(
        output.status.success(),
        "{name} runtime fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Runs one linked-workspace binary that must complete successfully.
fn assert_linked_fixture_succeeds(fixture_root: &Path, target_dir: &Path, binary: &str) {
    let output = run_linked_fixture(fixture_root, target_dir, binary);
    assert!(
        output.status.success(),
        "linked-workspace {binary} fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks the missing-runtime diagnostic without relying on the test crate's
/// development dependencies.
fn assert_missing_runtime_fixture_fails(fixture_root: &Path, target_dir: &Path) {
    let output = run_fixture(fixture_root, target_dir, "missing");
    assert!(!output.status.success(), "missing runtime fixture compiled");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Model derive requires the `qubit-model-metadata` dependency"),
        "unexpected missing runtime diagnostic: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Checks that a missing runtime dependency preserves independent validation
/// diagnostics from the same model declaration.
fn assert_missing_runtime_fixture_preserves_validation_error(fixture_root: &Path, target_dir: &Path) {
    let output = run_fixture(fixture_root, target_dir, "missing-invalid");
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
fn run_fixture(fixture_root: &Path, target_dir: &Path, name: &str) -> std::process::Output {
    let fixture_dir = fixture_root.join(name);
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
fn run_linked_fixture(fixture_root: &Path, target_dir: &Path, binary: &str) -> std::process::Output {
    let fixture_dir = fixture_root.join("linked-workspace");
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

/// Copies runtime fixtures into a disposable tree and resolves their local
/// dependency paths against the original checkout, including an optional
/// reflect integration worktree override.
fn prepare_runtime_fixtures(source_root: &Path, scratch: &Path) -> PathBuf {
    let destination_root = scratch.join("runtime-fixtures");
    copy_tree(source_root, &destination_root);
    let reflect_root = std::env::var_os("QUBIT_REFLECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| source_root.join("../../../../rs-reflect"));
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let validator_root = workspace_root.join("../../rust-common/rs-validator");
    rewrite_fixture_manifests(
        source_root,
        &destination_root,
        source_root,
        &destination_root,
        &reflect_root,
        &validator_root,
    );
    destination_root
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("fixture copy directory");
    for entry in fs::read_dir(source).expect("fixture directory") {
        let entry = entry.expect("fixture entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).expect("fixture file copy");
        }
    }
}

fn rewrite_fixture_manifests(
    source_dir: &Path,
    destination_dir: &Path,
    fixture_root: &Path,
    destination_root: &Path,
    reflect_root: &Path,
    validator_root: &Path,
) {
    for entry in fs::read_dir(source_dir).expect("fixture directory") {
        let entry = entry.expect("fixture entry");
        let source_path = entry.path();
        let destination_path = destination_dir.join(entry.file_name());
        if source_path.is_dir() {
            rewrite_fixture_manifests(
                &source_path,
                &destination_path,
                fixture_root,
                destination_root,
                reflect_root,
                validator_root,
            );
        } else if source_path.file_name().is_some_and(|name| name == "Cargo.toml") {
            let original = fs::read_to_string(&source_path).expect("fixture manifest");
            let rewritten = original
                .lines()
                .map(|line| {
                    let Some(path_start) = line.find("path = \"") else {
                        return line.to_owned();
                    };
                    let value_start = path_start + "path = \"".len();
                    let Some(value_end_offset) = line[value_start..].find('"') else {
                        return line.to_owned();
                    };
                    let value_end = value_start + value_end_offset;
                    let declared = Path::new(&line[value_start..value_end]);
                    let resolved = source_path
                        .parent()
                        .expect("manifest parent")
                        .join(declared)
                        .canonicalize()
                        .unwrap_or_else(|_| source_path.parent().expect("manifest parent").join(declared));
                    let fixture_root = fixture_root.canonicalize().expect("fixture source root");
                    let replacement = if resolved.ends_with("rs-reflect") {
                        reflect_root.to_path_buf()
                    } else if resolved.ends_with("rs-validator") {
                        validator_root.to_path_buf()
                    } else if resolved.starts_with(&fixture_root) {
                        destination_root.join(resolved.strip_prefix(fixture_root).expect("fixture-relative path"))
                    } else {
                        resolved
                    };
                    format!(
                        "{}{}{}",
                        &line[..value_start],
                        replacement.display(),
                        &line[value_end..]
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(destination_path, rewritten + "\n").expect("rewrite fixture manifest");
        }
    }
}
