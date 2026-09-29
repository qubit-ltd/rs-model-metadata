// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static positions of direct reflected child types.

/// The declaration relationship of a direct child type.
///
/// Container elements describe their element type, not a runtime element index.
/// Source Rust names and field query names remain separate facts; positional
/// fields retain absent names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StructuralEdgeKind {
    /// The element of an optional wrapper.
    OptionalElement,
    /// The pointee of a smart-pointer wrapper.
    PointerPointee,
    /// The element type of a sequence.
    SequenceElement,
    /// The element type of a set.
    SetElement,
    /// The element type of a fixed-length array.
    ArrayElement,
    /// The element type of an unsized slice.
    SliceElement,
    /// The key type of a map.
    MapKey,
    /// The value type of a map.
    MapValue,
    /// An element at a static tuple position.
    TupleElement {
        /// Zero-based element position in the tuple declaration.
        index: usize,
    },
    /// A struct field at its source declaration position.
    StructField {
        /// Zero-based field position in the struct declaration.
        index: usize,
        /// Source Rust identifier, or `None` for positional fields.
        name: Option<&'static str>,
        /// Reflected lookup identifier, or `None` for positional fields.
        query_name: Option<&'static str>,
    },
    /// An enum variant field at its source declaration position.
    EnumVariantField {
        /// Zero-based variant position in the enum declaration.
        variant_index: usize,
        /// Source Rust identifier of the variant.
        variant_name: &'static str,
        /// Zero-based field position in the variant declaration.
        field_index: usize,
        /// Source Rust identifier, or `None` for positional fields.
        field_name: Option<&'static str>,
        /// Reflected field lookup identifier, or `None` for positional fields.
        query_name: Option<&'static str>,
    },
}
