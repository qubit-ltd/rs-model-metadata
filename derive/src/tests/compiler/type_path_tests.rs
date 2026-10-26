// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests canonical standard-library type-path classification.

use syn::Path;
use syn::parse_quote;

use crate::compiler::type_path::is_collection_path;
use crate::compiler::type_path::is_option_path;
use crate::compiler::type_path::is_string_path;

/// Confirms canonical paths are accepted and lookalike module paths are
/// rejected.
#[test]
fn test_standard_type_path_classification() {
    let option: Path = parse_quote!(std::option::Option);
    let absolute_option: Path = parse_quote!(::core::option::Option);
    let fake_option: Path = parse_quote!(domain::Option);
    let vector: Path = parse_quote!(alloc::vec::Vec);
    let deque: Path = parse_quote!(std::collections::VecDeque);
    let hash_map: Path = parse_quote!(std::collections::HashMap);
    let tree_map: Path = parse_quote!(alloc::collections::BTreeMap);
    let qualified_vector: Path = parse_quote!(domain::collections::Vec);
    let fake_vector: Path = parse_quote!(domain::Vec);
    let string: Path = parse_quote!(std::string::String);
    let fake_string: Path = parse_quote!(domain::String);

    assert!(is_option_path(&option));
    assert!(is_option_path(&absolute_option));
    assert!(!is_option_path(&fake_option));
    assert!(is_collection_path(&vector));
    assert!(is_collection_path(&deque));
    assert!(is_collection_path(&hash_map));
    assert!(is_collection_path(&tree_map));
    assert!(!is_collection_path(&qualified_vector));
    assert!(!is_collection_path(&fake_vector));
    assert!(is_string_path(&string));
    assert!(!is_string_path(&fake_string));
}
