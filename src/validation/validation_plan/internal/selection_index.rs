// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Lookup from owned field-name sequences to bound field-rule occurrences.

use std::collections::HashMap;

use qubit_validator::PathSegment;
use qubit_validator::ValidationPath;

use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::internal::path_reader::path_for;

/// Extracts the exact field-name sequence used for selection keys.
///
/// # Parameters
///
/// * `path` - Compiled report path whose collection positions are ignored.
///
/// # Returns
///
/// The field names in their original order, without splitting field names on
/// punctuation.
fn field_names(path: &ValidationPath) -> Vec<String> {
    path.as_segments()
        .iter()
        .filter_map(|segment| match segment {
            PathSegment::Field(name) => Some((*name).to_owned()),
            _ => None,
        })
        .collect()
}

/// Indexes every field-rule occurrence by the field names in its report path.
///
/// Collection positions are omitted from keys to preserve prior matching
/// semantics. The report paths are retained so selection preparation can
/// reuse the paths created while the index is built.
pub(in crate::validation::validation_plan) struct SelectionIndex {
    /// Maps each complete field-name sequence to every matching binding index.
    occurrences: HashMap<Vec<String>, Vec<usize>>,
    /// Stores the report path computed for each binding during index creation.
    paths: Box<[ValidationPath]>,
}

impl SelectionIndex {
    /// Builds an index from ordered field-rule bindings.
    ///
    /// # Parameters
    ///
    /// * `bindings` - Field-rule occurrences in declaration execution order.
    ///
    /// # Returns
    ///
    /// An index retaining all occurrences whose field names have the same
    /// sequence, including occurrences that differ only by collection index.
    pub(in crate::validation::validation_plan) fn new(bindings: &[FieldRuleBinding]) -> Self {
        let paths = bindings.iter().map(|binding| path_for(binding.value())).collect();
        Self::from_paths(paths)
    }

    /// Groups cached report paths by their field-name sequence.
    ///
    /// # Parameters
    ///
    /// * `paths` - Report paths in field-rule execution order.
    ///
    /// # Returns
    ///
    /// An index containing every path occurrence and the original paths for
    /// later reporting.
    fn from_paths(paths: Vec<ValidationPath>) -> Self {
        let mut occurrences = HashMap::with_capacity(paths.len());
        for (binding_index, path) in paths.iter().enumerate() {
            let key = field_names(path);
            occurrences.entry(key).or_insert_with(Vec::new).push(binding_index);
        }
        Self {
            occurrences,
            paths: paths.into_boxed_slice(),
        }
    }

    /// Returns all indexed occurrences matching a borrowed field-name path.
    ///
    /// # Parameters
    ///
    /// * `segments` - Complete requested sequence of field names.
    ///
    /// # Returns
    ///
    /// The declaration-ordered occurrence indices, or `None` when no bound
    /// field rule has that path.
    #[must_use]
    #[inline]
    pub(in crate::validation::validation_plan) fn matching(&self, segments: &[String]) -> Option<&[usize]> {
        self.occurrences.get(segments).map(Vec::as_slice)
    }

    /// Returns the cached report path for a field-rule occurrence.
    ///
    /// # Parameters
    ///
    /// * `binding_index` - Position of the occurrence in the binding slice.
    ///
    /// # Returns
    ///
    /// The report path created for this binding while building the index.
    #[must_use]
    #[inline]
    pub(in crate::validation::validation_plan) fn path(&self, binding_index: usize) -> &ValidationPath {
        &self.paths[binding_index]
    }
}

#[cfg(test)]
mod tests {
    use qubit_validator::PathSegment;
    use qubit_validator::ValidationPath;

    use super::SelectionIndex;
    /// The reference implementation of the previous field-name comparison.
    fn reference_matching(segments: &[String], path: &ValidationPath) -> bool {
        segments
            .iter()
            .map(String::as_str)
            .eq(path.as_segments().iter().filter_map(|segment| match segment {
                qubit_validator::PathSegment::Field(name) => Some(*name),
                _ => None,
            }))
    }

    #[test]
    fn test_field_name_keys_match_the_linear_reference() {
        // Keep the previous predicate as an oracle for key extraction:
        // non-field segments are ignored and field boundaries are preserved.
        let path = ValidationPath::root()
            .with_field("items")
            .with_index(3)
            .with_field("name");
        let matching = ["items".to_owned(), "name".to_owned()];
        let dotted = ["items.name".to_owned()];
        assert!(reference_matching(&matching, &path));
        assert_eq!(super::field_names(&path), matching);
        assert!(!reference_matching(&dotted, &path));
        assert_eq!(super::field_names(&path), ["items", "name"]);
        assert!(!reference_matching(&["items".to_owned()], &path));
    }

    #[test]
    fn test_index_returns_all_occurrences_and_uses_borrowed_exact_keys() {
        let index = SelectionIndex::from_paths(vec![
            ValidationPath::root()
                .with_field("items")
                .with_index(1)
                .with_field("name"),
            ValidationPath::root()
                .with_field("items")
                .with_index(8)
                .with_field("name"),
            ValidationPath::root().with_field("items").with_field("other"),
        ]);
        let matching = ["items".to_owned(), "name".to_owned()];
        let reordered = ["name".to_owned(), "items".to_owned()];
        let dotted = ["items.name".to_owned()];
        let expected: Vec<_> = index
            .paths
            .iter()
            .enumerate()
            .filter_map(|(index, path)| reference_matching(&matching, path).then_some(index))
            .collect();
        assert_eq!(expected, [0, 1]);
        assert_eq!(index.matching(&matching), Some(expected.as_slice()));
        assert_eq!(index.matching(&reordered), None);
        assert_eq!(index.matching(&dotted), None);
        for (binding_index, expected) in [
            (
                0,
                ValidationPath::root()
                    .with_field("items")
                    .with_index(1)
                    .with_field("name"),
            ),
            (
                1,
                ValidationPath::root()
                    .with_field("items")
                    .with_index(8)
                    .with_field("name"),
            ),
        ] {
            assert_eq!(index.path(binding_index), &expected);
            let actual_fields: Vec<_> = index
                .path(binding_index)
                .as_segments()
                .iter()
                .filter_map(|segment| match segment {
                    PathSegment::Field(name) => Some((*name).to_owned()),
                    _ => None,
                })
                .collect();
            assert_eq!(actual_fields, matching);
            assert!(reference_matching(&matching, index.path(binding_index)));
        }
    }
}
