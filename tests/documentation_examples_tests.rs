// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execute Markdown manifests and source without supplying hidden dependencies.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use serde_json::Value;
use serde_json::from_slice;

const DOCUMENTS: &[&str] = &[
    "README.md",
    "README.zh_CN.md",
    "derive/README.md",
    "derive/README.zh_CN.md",
    "doc/user_guide.md",
    "doc/user_guide.zh_CN.md",
    "derive/doc/user_guide.md",
    "derive/doc/user_guide.zh_CN.md",
];

/// A fence retains its exact source and a stable installation/example mapping.
struct Fence {
    language: String,
    example: String,
    line: usize,
    source: String,
}

/// Drops only sources created exclusively by this test; build caches survive.
struct DocumentationFixture {
    directory: PathBuf,
}

impl Drop for DocumentationFixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.directory) {
            eprintln!("cannot remove {}: {error}", self.directory.display());
        }
    }
}

#[test]
fn test_bilingual_documentation_examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = create_fixture();
    for (document_index, document) in DOCUMENTS.iter().enumerate() {
        let markdown = fs::read_to_string(root.join(document)).expect("read document");
        let blocks = fences(&markdown, document);
        let manifests: BTreeMap<_, _> = blocks
            .iter()
            .filter(|block| block.language == "toml")
            .map(|block| (block.example.as_str(), block))
            .collect();
        assert_eq!(
            manifests.len(),
            blocks.iter().filter(|block| block.language == "toml").count(),
            "{document}: duplicate installation identifier"
        );
        let programs: Vec<_> = blocks.iter().filter(|block| block.language == "rust").collect();
        assert!(!programs.is_empty(), "{document}: no complete runnable program");
        for (program_index, program) in programs.iter().enumerate() {
            let (installation, _) = program.example.split_once('/').unwrap_or_else(|| {
                panic!(
                    "{document}:{} example {}: expected installation/source identifier",
                    program.line, program.example
                )
            });
            let manifest = manifests.get(installation).unwrap_or_else(|| {
                panic!(
                    "{document}:{} example {}: missing installation {installation}",
                    program.line, program.example
                )
            });
            let directory = fixture
                .directory
                .join("rs-platform")
                .join(format!("app-{document_index}-{program_index}"));
            let context = format!(
                "{document}:{} example {} (manifest line {})",
                program.line, program.example, manifest.line
            );
            prepare_consumer(&directory, root, &manifest.source, &program.source);
            let metadata = metadata(&directory);
            assert!(
                metadata.status.success(),
                "{context}: cargo metadata failed:\n{}",
                stderr(&metadata)
            );
            let metadata: Value = from_slice(&metadata.stdout)
                .unwrap_or_else(|error| panic!("{context}: invalid Cargo metadata JSON: {error}"));
            check_package_identity(&metadata, installation, &context);
            let output = cargo(&directory, &["run", "--locked", "--quiet"]);
            assert!(
                output.status.success(),
                "{context}: source execution failed:\n{}",
                stderr(&output)
            );
        }
    }
}

/// Removing a dependency must fail using the actual validation source. The old
/// fixture silently injected this dependency into one shared manifest.
#[test]
fn test_installation_rejects_missing_direct_dependency() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let blocks = guide_fences(root);
    let manifest = blocks
        .iter()
        .find(|block| block.example == "validation" && block.language == "toml")
        .expect("validation manifest");
    let program = blocks
        .iter()
        .find(|block| block.example == "validation/profile")
        .expect("validation source");
    let incomplete = manifest
        .source
        .lines()
        .filter(|line| !line.starts_with("qubit-reflect ="))
        .collect::<Vec<_>>()
        .join("\n");
    let fixture = create_fixture();
    let directory = fixture.directory.join("rs-platform/app-missing-reflect");
    prepare_consumer(&directory, root, &incomplete, &program.source);
    let output = cargo(&directory, &["check", "--quiet"]);
    assert!(
        !output.status.success(),
        "doc/user_guide.md:{} validation/profile: missing direct reflect dependency must fail",
        manifest.line
    );
    assert!(
        stderr(&output).contains("error[E0432]") && stderr(&output).contains("qubit_reflect"),
        "wrong failure: {}",
        stderr(&output)
    );
}

/// The validation program cannot borrow a feature from another example.
#[test]
fn test_installation_rejects_missing_validation_feature() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let blocks = guide_fences(root);
    let manifest = blocks
        .iter()
        .find(|block| block.example == "validation" && block.language == "toml")
        .expect("validation manifest");
    let program = blocks
        .iter()
        .find(|block| block.example == "validation/profile")
        .expect("validation source");
    let incomplete = manifest.source.replace(", features = [\"validation\"]", "");
    assert_ne!(incomplete, manifest.source, "actual manifest must declare validation");
    let fixture = create_fixture();
    let directory = fixture.directory.join("rs-platform/app-missing-feature");
    prepare_consumer(&directory, root, &incomplete, &program.source);
    let output = cargo(&directory, &["check", "--quiet"]);
    assert!(
        !output.status.success(),
        "doc/user_guide.md:{} validation/profile: missing validation feature must fail",
        manifest.line
    );
    assert!(
        stderr(&output).contains("could not find `validation` in `qubit_model_metadata`"),
        "wrong failure: {}",
        stderr(&output)
    );
}

/// Reads the English runtime guide used by both installation-failure
/// regressions and preserves each fence's manifest/source mapping and Markdown
/// location.
fn guide_fences(root: &Path) -> Vec<Fence> {
    fences(
        &fs::read_to_string(root.join("doc/user_guide.md")).expect("runtime guide"),
        "doc/user_guide.md",
    )
}

/// Markdown declares checkout roots; only those roots become absolute. Preserve
/// symlink spelling so direct and runtime dependencies share Cargo identities.
fn prepare_consumer(directory: &Path, root: &Path, manifest: &str, source: &str) {
    fs::create_dir_all(directory.join("src")).expect("consumer sources");
    let replacements = [
        ("../rs-model-metadata", root.to_path_buf()),
        ("../rs-reflect", root.join("../rs-reflect")),
        ("../../rust-common", root.join("../../rust-common")),
    ];
    let mut dependencies = manifest.to_owned();
    for (declared, actual) in replacements {
        assert!(actual.is_dir(), "checkout root {} exists", actual.display());
        let absolute = actual.to_str().expect("UTF-8 checkout root");
        assert!(!absolute.chars().any(char::is_control), "printable checkout path");
        dependencies = dependencies.replace(declared, &absolute.replace('\\', "\\\\").replace('"', "\\\""));
    }
    // Package/workspace scaffolding is independent of dependency configuration.
    let package = "[package]\nname = \"model-documentation-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n[workspace]\n\n";
    fs::write(directory.join("Cargo.toml"), format!("{package}{dependencies}")).expect("exact documented dependencies");
    fs::write(directory.join("src/main.rs"), source).expect("verbatim Markdown source");
}

/// Resolves the consumer's documented manifest and returns Cargo's
/// package/feature graph; resolution failure is returned without repairing it.
fn metadata(directory: &Path) -> Output {
    cargo(directory, &["metadata", "--format-version", "1"])
}

/// Every nested invocation uses a target distinct from the parent cargo test.
fn cargo(directory: &Path, arguments: &[&str]) -> Output {
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"))
        .join("documentation-consumers");
    Command::new(env!("CARGO"))
        .args(arguments)
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("nested Cargo invocation")
}

/// Retains Cargo/compiler diagnostics as printable text even when stderr
/// contains invalid UTF-8; callers attach the Markdown/example context.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Compare package IDs, including path identity, rather than version strings.
fn check_package_identity(metadata: &Value, installation: &str, context: &str) {
    let resolve = &metadata["resolve"];
    let nodes = resolve["nodes"].as_array().expect("resolved nodes");
    let consumer = nodes
        .iter()
        .find(|node| node["id"] == resolve["root"])
        .expect("consumer node");
    let runtime_id = dependency_id(consumer, "qubit_model_metadata")
        .unwrap_or_else(|| panic!("{context}: missing documented runtime dependency"));
    let runtime = nodes
        .iter()
        .find(|node| node["id"].as_str() == Some(runtime_id))
        .expect("runtime node");
    let features = runtime["features"].as_array().expect("runtime features");
    eprintln!("{context}: runtime features = {features:?}");
    assert_eq!(
        features.iter().any(|feature| feature == "validation"),
        installation == "validation",
        "{context}: independent runtime features"
    );
    for dependency in ["qubit_reflect", "qubit_validator", "qubit_id"] {
        if let Some(direct) = dependency_id(consumer, dependency) {
            assert_eq!(
                Some(direct),
                dependency_id(runtime, dependency),
                "{context}: direct/transitive {dependency} package ID differs"
            );
            eprintln!("{context}: {dependency} direct/transitive package ID = {direct}");
        } else if installation == "validation" && dependency != "qubit_id" {
            panic!("{context}: missing direct {dependency} dependency");
        }
    }
}

/// Looks up a resolved package ID by the consuming crate's dependency name,
/// preserving Cargo's path identity instead of comparing package versions.
fn dependency_id<'a>(node: &'a Value, name: &str) -> Option<&'a str> {
    node["deps"]
        .as_array()
        .expect("node dependencies")
        .iter()
        .find(|dependency| dependency["name"].as_str() == Some(name))
        .and_then(|dependency| dependency["pkg"].as_str())
}

/// Exclusive directory creation prevents adoption/deletion of another run.
fn create_fixture() -> DocumentationFixture {
    let parent = env::temp_dir().join("qubit-model-documentation-fixtures");
    fs::create_dir_all(&parent).expect("fixture parent");
    for index in 0..1_000 {
        let directory = parent.join(format!("{}-{index}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => return DocumentationFixture { directory },
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create documentation fixture: {error}"),
        }
    }
    panic!("fixture namespace exhausted");
}

/// Stable markers map each complete program to a real installation fence.
/// Ignored fragments are not promoted to complete programs.
fn fences(markdown: &str, document: &str) -> Vec<Fence> {
    let mut result = Vec::new();
    let mut marker = None;
    let mut active: Option<Fence> = None;
    for (index, line) in markdown.lines().enumerate() {
        if let Some(block) = active.as_mut() {
            if line == "```" {
                result.push(active.take().expect("active fence"));
            } else {
                block.source.push_str(line);
                block.source.push('\n');
            }
        } else if let Some(value) = line
            .strip_prefix("<!-- example: ")
            .and_then(|value| value.strip_suffix(" -->"))
        {
            marker = Some(value.to_owned());
        } else if let Some(language) = line.strip_prefix("```") {
            let executable = matches!(language, "rust" | "toml");
            let example = if executable {
                marker
                    .take()
                    .unwrap_or_else(|| panic!("{document}:{}: missing example marker", index + 1))
            } else {
                String::new()
            };
            active = Some(Fence {
                language: language.to_owned(),
                example,
                line: index + 2,
                source: String::new(),
            });
        }
    }
    assert!(active.is_none(), "{document}: unclosed fence");
    result
}
