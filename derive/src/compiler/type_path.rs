// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Classifies canonical standard-library type paths used by macro syntax.

use syn::Path;

/// Classifies paths written as `Option`, `std::option::Option`, or
/// `core::option::Option` as the standard `Option` type.
///
/// This is a syntactic match on the complete path; it does not resolve aliases
/// or verify that an unqualified `Option` refers to the standard library.
///
/// # Parameters
///
/// - `path`: The parsed type path to classify.
///
/// # Returns
///
/// `true` when the path has one of the supported forms, and `false` otherwise.
#[must_use]
#[inline]
pub(crate) fn is_option_path(path: &Path) -> bool {
    matches_path(path, &["Option"])
        || matches_path(path, &["std", "option", "Option"])
        || matches_path(path, &["core", "option", "Option"])
}

/// Classifies paths written as `String`, `std::string::String`, or
/// `alloc::string::String` as the standard `String` type.
///
/// This is a syntactic match on the complete path; it does not resolve aliases
/// or verify that an unqualified `String` refers to the standard library.
///
/// # Parameters
///
/// - `path`: The parsed type path to classify.
///
/// # Returns
///
/// `true` when the path has one of the supported forms, and `false` otherwise.
#[must_use]
#[inline]
pub(crate) fn is_string_path(path: &Path) -> bool {
    matches_path(path, &["String"])
        || matches_path(path, &["std", "string", "String"])
        || matches_path(path, &["alloc", "string", "String"])
}

/// Classifies supported standard collection paths whose types provide
/// `is_empty`.
///
/// Unqualified paths are matched by their final identifier, while qualified
/// paths must exactly match a supported `std` or `alloc` path. Other prefixes,
/// qualified suffixes, and differently named types are rejected.
///
/// # Parameters
///
/// - `path`: The parsed type path to classify.
///
/// # Returns
///
/// `true` when the path matches a supported collection form, and `false`
/// otherwise.
#[must_use]
#[inline]
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
///
/// The path must have the same number and order of segments; leading or
/// trailing qualification is not ignored.
///
/// # Parameters
///
/// - `path`: The parsed path to compare.
/// - `expected`: The expected identifiers in path order.
///
/// # Returns
///
/// `true` only when every path segment matches its corresponding identifier.
#[must_use]
#[inline]
fn matches_path(path: &Path, expected: &[&str]) -> bool {
    path.segments.len() == expected.len()
        && path
            .segments
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.ident == *expected)
}

/// Matches an exact `std::collections::Name` or `alloc::collections::Name`
/// path using the supplied collection name.
///
/// # Parameters
///
/// - `path`: The parsed path to compare.
/// - `root`: The expected `std` or `alloc` root segment.
/// - `name`: The expected collection type segment.
///
/// # Returns
///
/// `true` when all path segments match the expected collection path.
#[must_use]
#[inline]
fn matches_collection_path(path: &Path, root: &str, name: &str) -> bool {
    matches_path(path, &[root, "collections", name])
}
