// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::ValueStringCodecRegistry;

use crate::resolve::ModelGraph;

/// Inputs for one codec binding pass.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the model registry borrowed by the graph.
/// * `'graph` - Lifetime of the graph and codec registry borrows.
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
/// let context = ReflectionContext::builder().build()?;
/// let models = ModelRegistry::from_static_metadata(&context, &[])?;
/// let roots: &[&'static qubit_model_metadata::metadata::TypeMetadata] = &[];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots }).resolve()?;
/// let codecs = ValueStringCodecRegistry::empty();
/// let bindings = bind_codecs(CodecBindInputs { graph: &graph, codecs: &codecs })?;
/// assert_eq!(bindings.bindings().len(), 0);
/// # Ok(())
/// # }
/// ```
pub struct CodecBindInputs<'a, 'graph> {
    /// Structure graph containing codec declarations.
    pub graph: &'graph ModelGraph<'a>,
    /// Executable codec registry.
    pub codecs: &'graph ValueStringCodecRegistry,
}
