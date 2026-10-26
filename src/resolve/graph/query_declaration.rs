// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One indexed member declaration retained for downstream query generation.

use crate::metadata::FieldMetadata;
use crate::metadata::IndexingReasons;
use crate::metadata::PropertyPath;
use crate::resolve::internal::OwnedPropertyPath;

/// One declared indexed member and the facts making it indexed.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::QueryDeclaration;
/// use qubit_model_metadata::resolve::QueryMetadata;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let declarations: &[QueryDeclaration] = graph
///     .query(root.type_id())
///     .map(QueryMetadata::declarations)
///     .unwrap_or(&[]);
/// assert!(declarations.is_empty());
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct QueryDeclaration {
    /// Original field carrying type, uniqueness and reference metadata.
    pub(in crate::resolve) field: &'static FieldMetadata,
    /// Direct member path relative to the declaring Entity.
    pub(in crate::resolve) path: OwnedPropertyPath,
}

impl QueryDeclaration {
    /// Returns the original declaration for further metadata navigation.
    ///
    /// # Returns
    ///
    /// Static field metadata carrying this indexed member's declaration facts.
    #[must_use]
    #[inline]
    pub const fn field(&self) -> &'static FieldMetadata {
        self.field
    }

    /// Returns the direct property path in declaration order.
    ///
    /// # Returns
    ///
    /// The owned path viewed as borrowed segments in declaration order.
    #[must_use]
    #[inline]
    pub fn path(&self) -> PropertyPath<'_> {
        self.path.as_path()
    }

    /// Returns all explicit and implicit indexing reasons.
    ///
    /// # Returns
    ///
    /// The reasons that caused this field to be included in query metadata.
    #[must_use]
    #[inline]
    pub fn reasons(&self) -> IndexingReasons {
        self.field.indexing_reasons()
    }
}
