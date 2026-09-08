// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Classifies canonical standard-library type paths used by macro syntax.

use syn::Path;

/// Returns whether `path` names the standard `Option` type.
pub(crate) fn is_option_path(path: &Path) -> bool {
    matches_path(path, &["Option"])
        || matches_path(path, &["std", "option", "Option"])
        || matches_path(path, &["core", "option", "Option"])
}

/// Returns whether `path` names the standard `String` type.
pub(crate) fn is_string_path(path: &Path) -> bool {
    matches_path(path, &["String"])
        || matches_path(path, &["std", "string", "String"])
        || matches_path(path, &["alloc", "string", "String"])
}

/// Returns whether `path` names a standard collection with `is_empty`.
pub(crate) fn is_collection_path(path: &Path) -> bool {
    let Some(name) = path.segments.last().map(|segment| segment.ident.to_string()) else {
        return false;
    };
    if path.segments.len() == 1 {
        return matches!(
            name.as_str(),
            "Vec" | "VecDeque" | "LinkedList" | "BinaryHeap" | "HashSet" | "BTreeSet" | "HashMap" | "BTreeMap"
        );
    }
    match name.as_str() {
        "Vec" => matches_path(path, &["std", "vec", "Vec"]) || matches_path(path, &["alloc", "vec", "Vec"]),
        "HashSet" | "HashMap" => matches_collection_path(path, "std", &name),
        "VecDeque" | "LinkedList" | "BinaryHeap" | "BTreeSet" | "BTreeMap" => {
            matches_collection_path(path, "std", &name) || matches_collection_path(path, "alloc", &name)
        }
        _ => false,
    }
}

/// Compares a Syn path with an exact list of identifier segments.
fn matches_path(path: &Path, expected: &[&str]) -> bool {
    path.segments.len() == expected.len()
        && path
            .segments
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.ident == *expected)
}

/// Matches `std::collections::Name` or `alloc::collections::Name`.
fn matches_collection_path(path: &Path, root: &str, name: &str) -> bool {
    matches_path(path, &[root, "collections", name])
}

#[cfg(test)]
mod tests {
    use syn::Path;
    use syn::parse_quote;

    use super::is_collection_path;
    use super::is_option_path;
    use super::is_string_path;

    /// Confirms canonical paths are accepted and lookalike module paths are
    /// rejected.
    #[test]
    fn test_standard_type_path_classification() {
        let option: Path = parse_quote!(std::option::Option);
        let absolute_option: Path = parse_quote!(::core::option::Option);
        let fake_option: Path = parse_quote!(domain::Option);
        let vector: Path = parse_quote!(alloc::vec::Vec);
        let fake_vector: Path = parse_quote!(domain::Vec);
        let string: Path = parse_quote!(std::string::String);
        let fake_string: Path = parse_quote!(domain::String);

        assert!(is_option_path(&option));
        assert!(is_option_path(&absolute_option));
        assert!(!is_option_path(&fake_option));
        assert!(is_collection_path(&vector));
        assert!(!is_collection_path(&fake_vector));
        assert!(is_string_path(&string));
        assert!(!is_string_path(&fake_string));
    }
}
