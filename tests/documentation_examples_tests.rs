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

/// Resolves the published installation fences without rewriting any path.
#[test]
fn test_documented_installation_paths() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent = root.parent().expect("metadata checkout parent");
    let mut configurations = Vec::new();
    let mut installation_kinds = BTreeMap::new();
    for document in DOCUMENTS {
        let markdown = fs::read_to_string(root.join(document)).expect("read document");
        for block in fences(&markdown, document) {
            if ["core", "core/model-id", "validation", "codec"].contains(&block.example.as_str())
                && block.language == "toml"
            {
                installation_kinds.insert(block.example.clone(), ());
                configurations.push((block.example, (*document).to_owned(), block.line, block.source));
            }
        }
    }

    assert_eq!(
        installation_kinds.len(),
        4,
        "expected core, model-id, validation, and codec installation fences"
    );
    for (index, (installation, document, line, dependencies)) in configurations.iter().enumerate() {
        let directory = parent.join(format!("app-documentation-{}-{index}", std::process::id()));
        let fixture = create_fixture_at(&directory);
        write_raw_consumer(&fixture.directory, dependencies);
        let output = metadata(&fixture.directory);
        let context = format!("{document}:{line} {installation} raw installation");
        assert!(
            output.status.success(),
            "{context}: cargo metadata failed:\n{}",
            stderr(&output)
        );
        let metadata: Value = from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{context}: invalid Cargo metadata JSON: {error}"));
        check_raw_installation_identity(&metadata, installation, root, parent, &context);
    }

    let core = configurations
        .iter()
        .find(|(installation, _, _, _)| installation == "core")
        .map(|(_, _, _, dependencies)| dependencies)
        .expect("core installation");
    let negative_root = parent.join(format!("documentation-negative-{}", std::process::id()));
    let fixture = create_fixture_at(&negative_root);
    let directory = fixture.directory.join("rs-platform").join("app");
    write_raw_consumer(&directory, core);
    let output = metadata(&directory);
    let expected_missing = directory
        .parent()
        .expect("negative app parent")
        .join("rs-model-metadata");
    let diagnostic = stderr(&output);
    assert!(
        !output.status.success(),
        "rs-platform/app unexpectedly resolved raw paths"
    );
    assert!(
        diagnostic.contains(&expected_missing.display().to_string())
            && (diagnostic.contains("does not exist") || diagnostic.contains("failed to read")),
        "rs-platform/app failure did not identify its incorrect rs-model-metadata path ({}): {diagnostic}",
        expected_missing.display()
    );
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
    let diagnostic = stderr(&output);
    assert!(
        diagnostic.contains("cannot resolve `qubit-reflect`")
            || diagnostic.contains("error[E0432]") && diagnostic.contains("qubit_reflect"),
        "wrong failure: {diagnostic}"
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
    let reflect_root = env::var_os("QUBIT_REFLECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("../rs-reflect"));
    let replacements = [
        ("../rs-model-metadata", root.to_path_buf()),
        ("../rs-reflect", reflect_root),
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
    let package = concat!(
        "[package]\n",
        "name = \"model-documentation-consumer\"\n",
        "version = \"0.0.0\"\n",
        "edition = \"2024\"\n",
        "publish = false\n\n",
        "[workspace]\n\n",
    );
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

fn check_raw_installation_identity(metadata: &Value, installation: &str, root: &Path, parent: &Path, context: &str) {
    let documented_root = if root.file_name().is_some_and(|name| name == "rs-model-metadata") {
        root.to_path_buf()
    } else {
        parent.join("rs-model-metadata")
    };
    let documented_root = documented_root.canonicalize().unwrap_or(documented_root);
    let expected: &[(&str, PathBuf)] = match installation {
        "core" | "validation" | "codec" => &[
            ("qubit-model-metadata", documented_root.clone()),
            ("qubit-model-derive", documented_root.join("derive")),
            ("qubit-model-id", documented_root.join("model-id")),
            ("qubit-reflect", parent.join("rs-reflect")),
        ],
        "core/model-id" => &[("qubit-model-id", documented_root.join("model-id"))],
        other => panic!("{context}: unexpected installation {other}"),
    };
    let packages = metadata["packages"].as_array().expect("metadata packages");
    for (name, directory) in expected {
        let manifest = directory.join("Cargo.toml");
        let package = packages.iter().find(|package| package["name"] == *name);
        let package = package.unwrap_or_else(|| panic!("{context}: missing package {name}"));
        assert_eq!(
            package["manifest_path"].as_str(),
            manifest.canonicalize().unwrap_or(manifest).to_str(),
            "{context}: {name} checkout identity"
        );
    }
    if installation == "validation" {
        let manifest = parent
            .parent()
            .expect("checkout root")
            .join("rust-common/rs-validator/Cargo.toml");
        let package = packages
            .iter()
            .find(|package| package["name"] == "qubit-validator")
            .expect("validation installation validator package");
        assert_eq!(
            package["manifest_path"].as_str(),
            manifest.canonicalize().unwrap_or(manifest).to_str(),
            "{context}: validator checkout identity"
        );
    }
    if matches!(installation, "core" | "validation" | "codec") {
        let nodes = metadata["resolve"]["nodes"].as_array().expect("resolved nodes");
        let runtime = nodes
            .iter()
            .find(|node| {
                node["id"]
                    .as_str()
                    .is_some_and(|id| id.contains("qubit-model-metadata"))
            })
            .expect("runtime resolve node");
        let features = runtime["features"].as_array().expect("runtime features");
        assert_eq!(
            features.iter().any(|feature| feature == "validation"),
            installation == "validation",
            "{context}: validation feature"
        );
        assert_eq!(
            features.iter().any(|feature| feature == "codec"),
            installation == "codec",
            "{context}: codec feature"
        );
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

/// Claims one exact path exclusively so its drop guard owns all cleanup.
fn create_fixture_at(directory: &Path) -> DocumentationFixture {
    fs::create_dir(directory).unwrap_or_else(|error| {
        panic!(
            "create exclusive documentation fixture {}: {error}",
            directory.display()
        )
    });
    DocumentationFixture {
        directory: directory.to_path_buf(),
    }
}

/// Writes only package scaffolding and the unmodified TOML dependency fence.
fn write_raw_consumer(directory: &Path, dependencies: &str) {
    fs::create_dir_all(directory.join("src")).expect("raw consumer sources");
    let package = concat!(
        "[package]\n",
        "name = \"model-documentation-raw-consumer\"\n",
        "version = \"0.0.0\"\n",
        "edition = \"2024\"\n",
        "publish = false\n\n",
        "[workspace]\n\n",
    );
    fs::write(directory.join("Cargo.toml"), format!("{package}{dependencies}")).expect("raw documented dependencies");
    fs::write(directory.join("src/main.rs"), "fn main() {}\n").expect("raw consumer source");
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
