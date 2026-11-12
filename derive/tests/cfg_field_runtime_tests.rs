// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Independent consumer builds for cfg-controlled declaration fields.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

mod support;
use support::temporary_target_dir::TemporaryTargetDir;

#[test]
fn test_cfg_field_feature_matrix() {
    let scratch = TemporaryTargetDir::new("qubit-model-cfg-fields-fixture");
    let manifest = prepare_fixture(scratch.path());
    let target = TemporaryTargetDir::new("qubit-model-cfg-fields-target");
    for features in [
        "test-extra,enum-probe",
        "test-extra,enum-probe,field-enabled",
        "test-extra,enum-probe,via-cfg-attr",
        "test-extra,enum-probe,field-enabled,via-cfg-attr",
        "test-extra,enum-probe,zero-field",
        "test-extra,enum-probe,zero-field,via-cfg-attr",
        "test-extra,enum-probe,bound-enabled",
    ] {
        for action in ["check", "test"] {
            let output = run(&manifest, target.path(), action, features);
            assert!(
                output.status.success(),
                "{action} {features}:\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
    let output = run(&manifest, target.path(), "check", "invalid-enabled");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "invalid enabled declaration unexpectedly compiled"
    );
    assert!(
        stderr.contains("Value") || stderr.contains("tuple"),
        "unexpected diagnostic: {stderr}"
    );

    let output = run(
        &manifest,
        target.path(),
        "check",
        "test-extra,bound-enabled,expect-bound-failure",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "conditional Clone bound unexpectedly disappeared"
    );
    assert!(
        stderr.contains("Clone"),
        "unexpected conditional-bound diagnostic: {stderr}"
    );

    let output = run(
        &manifest,
        target.path(),
        "check",
        "test-extra,transparent-disabled,expect-transparent-failure",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "non-Display transparent field unexpectedly compiled"
    );
    assert!(
        stderr.contains("Display"),
        "unexpected transparent-bound diagnostic: {stderr}"
    );

    let output = run(&manifest, target.path(), "check", "tuple-limit");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "oversized tuple cfg shape unexpectedly compiled"
    );
    assert!(
        stderr.contains("conditional tuple"),
        "unexpected tuple-limit diagnostic: {stderr}"
    );
}

/// Copies the fixture and rewrites local dependencies to the active checkouts.
///
/// # Parameters
/// * `directory`: temporary parent directory for the copied fixture.
///
/// # Returns
/// The manifest path of the prepared fixture.
fn prepare_fixture(directory: &Path) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("tests/runtime-fixtures/cfg-fields");
    let destination = directory.join("cfg-fields");
    fs::create_dir_all(destination.join("src")).unwrap();
    fs::copy(source.join("src/lib.rs"), destination.join("src/lib.rs")).unwrap();
    fs::copy(source.join("Cargo.lock"), destination.join("Cargo.lock")).unwrap();
    let reflect = std::env::var_os("QUBIT_REFLECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("../../rs-reflect"));
    let manifest = fs::read_to_string(source.join("Cargo.toml")).unwrap()
        .replace("qubit-model-derive = { path = \"../../../\", version = \"^0.2\" }", &format!("qubit-model-derive = {{ path = \"{}\", version = \"^0.2\" }}", root.display()))
        .replace("model_runtime = { package = \"qubit-model-metadata\", path = \"../../../../\", features = [\"generic\"] , version = \"^0.2\" }", &format!("model_runtime = {{ package = \"qubit-model-metadata\", path = \"{}\", features = [\"generic\"], version = \"^0.2\" }}", root.join("..").display()))
        .replace("qubit-reflect = { path = \"../../../../../rs-reflect\", version = \"^0.2\" }", &format!("qubit-reflect = {{ path = \"{}\", version = \"^0.2\" }}", reflect.display()));
    fs::write(destination.join("Cargo.toml"), manifest).unwrap();
    destination.join("Cargo.toml")
}

/// Runs a locked fixture Cargo action with the requested feature set.
///
/// # Parameters
/// * `manifest`: the fixture manifest to execute.
/// * `target`: the isolated Cargo target directory.
/// * `action`: Cargo action such as `check` or `test`.
/// * `features`: comma-separated features enabled for the run.
///
/// # Returns
/// The captured Cargo process output.
fn run(manifest: &Path, target: &Path, action: &str, features: &str) -> Output {
    let mut command = Command::new(env!("CARGO"));
    command
        .args([action, "--offline", "--locked", "--manifest-path"])
        .arg(manifest)
        .env("CARGO_TARGET_DIR", target);
    if !features.is_empty() {
        command.args(["--features", features]);
    }
    command.output().expect("fixture cargo process")
}
