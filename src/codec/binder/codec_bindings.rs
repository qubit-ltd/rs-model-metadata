// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::BTreeMap;

use super::codec_binding::CodecBinding;
use super::codec_occurrence_id::CodecOccurrenceId;

/// Immutable successful codec bindings keyed by stable occurrence identity.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the codec registry entries referenced by the bindings.
///
/// # Examples
///
/// ```
/// #![cfg(feature = "codec")]
/// use std::error::Error;
///
/// use qubit_codec::ValueStringCodecRegistry;
/// use qubit_model_metadata::codec::{CodecBindInputs, bind_codecs};
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};
/// use qubit_reflect::ReflectionContext;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let reflection = ReflectionContext::builder().build()?;
/// let models = ModelRegistry::from_static_metadata(&reflection, &[])?;
/// let roots: &[&'static qubit_model_metadata::metadata::TypeMetadata] = &[];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots }).resolve()?;
/// let codecs = ValueStringCodecRegistry::empty();
/// let bindings = bind_codecs(CodecBindInputs { graph: &graph, codecs: &codecs })?;
/// assert_eq!(bindings.bindings().len(), 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CodecBindings<'a>(
    /// Bindings indexed by stable occurrence identity.
    pub(super) BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
);

impl<'a> CodecBindings<'a> {
    /// Returns a binding by stable occurrence identity.
    ///
    /// # Parameters
    ///
    /// * `occurrence` - Stable identity of the declaration to look up.
    ///
    /// # Returns
    ///
    /// The matching binding, or `None` when the identity is not present.
    #[inline]
    pub fn get(&self, occurrence: &CodecOccurrenceId) -> Option<&CodecBinding<'a>> {
        self.0.get(occurrence)
    }

    /// Iterates over bindings in stable identity order.
    ///
    /// # Returns
    ///
    /// An exact-size iterator over bindings, ordered by occurrence identity.
    #[inline]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &CodecBinding<'a>> {
        self.0.values()
    }
}
