// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compile and execute the actual bilingual README and guide examples.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Owns only the newly created fixture sources; compiled dependencies stay
/// cached.
struct DocumentationFixture {
    directory: PathBuf,
}

impl Drop for DocumentationFixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.directory) {
            eprintln!(
                "cannot remove documentation fixture {}: {error}",
                self.directory.display()
            );
        }
    }
}

/// Builds each complete Rust fence as a separate application, then runs its
/// assertions. Separate binaries prevent unrelated model registrations from
/// changing the example's registry. Syntax-only declaration fragments marked
/// `rust,ignore` are deliberately outside this executable-example contract.
#[test]
fn test_bilingual_documentation_examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let documents = [
        "README.md",
        "README.zh_CN.md",
        "derive/README.md",
        "derive/README.zh_CN.md",
        "doc/user_guide.md",
        "doc/user_guide.zh_CN.md",
        "derive/doc/user_guide.md",
        "derive/doc/user_guide.zh_CN.md",
    ];
    let fixture_parent = root.join("target/documentation-fixtures");
    fs::create_dir_all(&fixture_parent).expect("documentation fixture parent");
    let fixture = create_fixture(&fixture_parent);
    let source_dir = fixture.directory.join("src/bin");
    fs::create_dir_all(&source_dir).expect("documentation binary directory");
    let mut examples = Vec::new();
    for (document_index, document) in documents.iter().enumerate() {
        let markdown = fs::read_to_string(root.join(document)).expect("document exists");
        let blocks = rust_blocks(&markdown);
        assert!(!blocks.is_empty(), "{document} must have a complete runnable example");
        for (block_index, (line, source)) in blocks.into_iter().enumerate() {
            let binary = format!("document_{document_index}_{block_index}");
            fs::write(source_dir.join(format!("{binary}.rs")), source).expect("write exact Markdown Rust block");
            examples.push((binary, format!("{document}:{line}")));
        }
    }

    // This isolated workspace uses the same checkout identities as the README.
    // The target directory differs from the parent test build to avoid nested
    // Cargo contention, while successive feature-matrix runs reuse artifacts.
    let manifest = format!(
        "[package]\nname = \"model-documentation-examples\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n\
         [workspace]\n\n[dependencies]\n\
         qubit-model-metadata = {{ path = {}, features = [\"validation\"] }}\n\
         qubit-model-derive = {{ path = {} }}\n\
         qubit-reflect = {{ path = {} }}\n\
         qubit-id = {{ path = {} }}\n\
         qubit-validator = {{ path = {} }}\n",
        toml_path(root),
        toml_path(&root.join("derive")),
        toml_path(&root.join("../rs-reflect")),
        toml_path(&root.join("../../rust-common/rs-id")),
        toml_path(&root.join("../../rust-common/rs-validator")),
    );
    fs::write(fixture.directory.join("Cargo.toml"), manifest).expect("documentation manifest");
    let target = root.join("target/documentation-examples");
    let output = Command::new(env!("CARGO"))
        .args(["build", "--offline", "--quiet", "--bins"])
        .current_dir(&fixture.directory)
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .expect("start documentation example compilation");
    assert!(
        output.status.success(),
        "Markdown example compilation failed; binary/source mapping: {examples:?}\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    for (binary, source) in examples {
        let output = Command::new(
            target
                .join("debug")
                .join(format!("{binary}{}", std::env::consts::EXE_SUFFIX)),
        )
        .output()
        .expect("run compiled documentation example");
        assert!(
            output.status.success(),
            "{source} example failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

/// Uses exclusive directory creation, never adopting or deleting an existing
/// fixture left by a concurrent run or a previous process with the same PID.
fn create_fixture(parent: &Path) -> DocumentationFixture {
    for index in 0..1_000 {
        let directory = parent.join(format!("{}-{index}", std::process::id()));
        match fs::create_dir(&directory) {
            Ok(()) => return DocumentationFixture { directory },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("cannot create documentation fixture: {error}"),
        }
    }
    panic!("documentation fixture namespace exhausted");
}

/// Quotes a checked filesystem path as a TOML basic string without invoking a
/// shell. Newlines are rejected because checkout paths are expected to be
/// ordinary printable filesystem paths.
fn toml_path(path: &Path) -> String {
    // Preserve symlink spelling: Cargo distinguishes the worktree's dependency
    // path from its canonical target when constructing package identities.
    assert!(path.is_dir(), "documentation dependency exists");
    let value = path.to_str().expect("UTF-8 checkout path");
    assert!(
        !value.chars().any(char::is_control),
        "unsupported control character in checkout path"
    );
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Extracts complete `rust` fences verbatim, retaining their Markdown line for
/// diagnostics. Other fenced languages and explicitly ignored field fragments
/// remain prose examples and are not silently rewritten into working programs.
fn rust_blocks(markdown: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut fence = None;
    let mut start = 0;
    let mut source = String::new();
    for (index, line) in markdown.lines().enumerate() {
        if let Some(language) = line.strip_prefix("```") {
            if let Some(is_rust) = fence.take() {
                assert!(
                    language.is_empty(),
                    "nested or unclosed Markdown fence at line {}",
                    index + 1
                );
                if is_rust {
                    blocks.push((start, std::mem::take(&mut source)));
                }
            } else {
                fence = Some(language == "rust");
                start = index + 2;
            }
        } else if fence == Some(true) {
            source.push_str(line);
            source.push('\n');
        }
    }
    assert!(fence.is_none(), "unclosed Markdown fence");
    blocks
}
