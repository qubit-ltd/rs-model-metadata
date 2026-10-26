// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Constructors shared by generated model code.

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;

use crate::metadata;

/// Wraps a successfully validated merged property slice.
///
/// # Parameters
///
/// - `properties`: Static slice validated during metadata construction.
///
/// # Returns
///
/// A local property set borrowing the supplied slice.
#[doc(hidden)]
#[must_use]
#[inline]
pub const fn local_property_set(properties: &'static [metadata::PropertyMetadata]) -> metadata::LocalPropertySet {
    metadata::LocalPropertySet::new(properties)
}

/// Creates one generated field/getter/setter source fragment.
///
/// # Parameters
///
/// - `name`: Logical property name.
/// - `type_expression`: `Some` records the declared type expression; `None`
///   leaves type resolution to other metadata.
/// - `source`: Origin of the generated fragment.
///
/// # Returns
///
/// A property fragment containing the supplied source information.
#[doc(hidden)]
#[must_use]
#[inline]
pub const fn property_fragment(
    name: &'static str,
    type_expression: Option<&'static TypeExpressionData>,
    source: metadata::PropertyFragmentSource,
) -> metadata::PropertyFragment {
    metadata::PropertyFragment::new(name, type_expression, source)
}

/// Creates the generated metadata attached by one `ModelImpl` block.
///
/// # Parameters
///
/// - `fragments`: Static generated accessor fragments.
/// - `properties`: `Ok` contains the validated local property set; `Err`
///   contains the property build errors retained for later reporting.
///
/// # Returns
///
/// Metadata associating the fragments with the property build outcome.
#[doc(hidden)]
#[must_use]
#[inline]
pub const fn model_impl_metadata(
    fragments: &'static [metadata::PropertyFragment],
    properties: Result<&'static metadata::LocalPropertySet, &'static metadata::PropertyBuildErrors>,
) -> metadata::ModelImplMetadata {
    metadata::ModelImplMetadata::new(fragments, properties)
}
