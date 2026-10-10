// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit codec binding over an immutable model graph.

use core::any::TypeId;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use std::collections::BTreeMap;

use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;
use qubit_reflect::TypeDescriptor;

use super::CodecBindError;
use super::CodecBindErrorKind;
use super::CodecBindErrors;
use crate::metadata::CodecMetadata;
use crate::metadata::CodecReference;
use crate::metadata::CodecSource;
use crate::metadata::FieldMetadata;
use crate::metadata::ModelIdBuf;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::transparent_descriptor::transparent_descriptor;

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
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let models = ModelRegistry::from_static_metadata(&[])?;
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

/// Stable identity of one codec occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecOccurrenceId;
/// use qubit_model_metadata::metadata::CodecSource;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.codec.Note")]
/// pub struct Note { pub title: String }
/// # }
///
/// let occurrence = CodecOccurrenceId::new(
///     TypeMetadata::of::<example::Note>(),
///     "title",
///     CodecSource::Field,
/// );
/// assert_eq!(occurrence.property(), "title");
/// assert_eq!(occurrence.source(), CodecSource::Field);
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CodecOccurrenceId {
    model: Option<ModelIdBuf>,
    type_id: TypeId,
    type_name: &'static str,
    property: Box<str>,
    source: CodecSource,
}

impl CodecOccurrenceId {
    /// Creates a stable occurrence identity.
    ///
    /// # Parameters
    ///
    /// * `model` - Static metadata for the owning model.
    /// * `property` - Normalized property path, or an empty string for a
    ///   canonical codec.
    /// * `source` - Declaration location within the model metadata.
    ///
    /// # Returns
    ///
    /// An identity containing the model type, property path, and declaration
    /// source.
    #[must_use]
    pub fn new(model: &'static TypeMetadata, property: impl Into<Box<str>>, source: CodecSource) -> Self {
        Self {
            model: model.model_id().map(ModelIdBuf::from),
            type_id: model.type_id(),
            type_name: model.type_name(),
            property: property.into(),
            source,
        }
    }

    /// Returns the owning model ID.
    ///
    /// # Returns
    ///
    /// `Some` with the stable ID when the model has one, or `None` for an
    /// anonymous model.
    #[must_use]
    #[inline]
    pub const fn model(&self) -> Option<&ModelIdBuf> {
        self.model.as_ref()
    }

    /// Returns the exact owner identity, including anonymous types.
    ///
    /// # Returns
    ///
    /// The concrete Rust type identity used for exact registration matching.
    #[must_use]
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the normalized property path, empty for canonical value codecs.
    ///
    /// # Returns
    ///
    /// The normalized path, or an empty string for a model's canonical value
    /// codec.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> &str {
        &self.property
    }

    /// Returns the declaration source.
    ///
    /// # Returns
    ///
    /// The model field, canonical value, or nested selector that supplied the
    /// codec.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}

impl Display for CodecOccurrenceId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        if self.property.is_empty() {
            write!(formatter, "{}::<canonical>", self.type_name)
        } else {
            write!(formatter, "{}::{}", self.type_name, self.property)
        }
    }
}

/// One codec declaration bound to an executable descriptor.
///
/// # Examples
///
/// ```
/// #![cfg(feature = "codec")]
/// use qubit_model_metadata::codec::CodecBinding;
///
/// fn inspect(binding: &CodecBinding<'_>) {
///     let _ = binding.occurrence();
///     let _ = binding.declaration();
///     let _ = binding.descriptor();
///     let _ = binding.registration();
/// }
/// # let _ = inspect;
/// ```
#[derive(Debug)]
pub struct CodecBinding<'a> {
    /// Stable identity used to locate this binding in the resolved graph.
    occurrence: CodecOccurrenceId,
    /// Model declaration that selected the codec.
    declaration: &'static CodecMetadata,
    /// Executable codec descriptor used for dispatch.
    descriptor: &'static ValueStringCodecDescriptor,
    /// Registry entry selected after uniqueness and type checks.
    registration: &'a ValueStringCodecRegistration,
}

impl<'a> CodecBinding<'a> {
    /// Returns the occurrence identity.
    ///
    /// # Returns
    ///
    /// The stable identity of the model declaration represented by this
    /// binding.
    #[must_use]
    #[inline]
    pub const fn occurrence(&self) -> &CodecOccurrenceId {
        &self.occurrence
    }

    /// Returns the declaration metadata.
    ///
    /// # Returns
    ///
    /// Static metadata for the codec declaration that selected this
    /// registration.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static CodecMetadata {
        self.declaration
    }

    /// Returns the executable descriptor.
    ///
    /// # Returns
    ///
    /// The descriptor used to dispatch values to the selected codec.
    #[must_use]
    #[inline]
    pub const fn descriptor(&self) -> &'static ValueStringCodecDescriptor {
        self.descriptor
    }

    /// Returns the selected registration.
    ///
    /// # Returns
    ///
    /// The registry entry selected after uniqueness and value-type checks.
    #[must_use]
    #[inline]
    pub const fn registration(&self) -> &'a ValueStringCodecRegistration {
        self.registration
    }
}

/// Immutable successful codec bindings keyed by stable occurrence identity.
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
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let models = ModelRegistry::from_static_metadata(&[])?;
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
    BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
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
    #[must_use]
    #[inline]
    pub fn get(&self, occurrence: &CodecOccurrenceId) -> Option<&CodecBinding<'a>> {
        self.0.get(occurrence)
    }

    /// Iterates over bindings in stable identity order.
    ///
    /// # Returns
    ///
    /// An exact-size iterator over bindings, ordered by occurrence identity.
    #[must_use]
    #[inline]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &CodecBinding<'a>> {
        self.0.values()
    }
}

/// Binds every codec declaration in a graph to an executable registration.
///
/// # Errors
///
/// Returns all missing, ambiguous, and value-type mismatched occurrences.
///
/// # Parameters
///
/// * `inputs` - The resolved model graph and executable codec registry.
///
/// # Returns
///
/// All successful bindings, keyed by stable codec occurrence identity.
pub fn bind_codecs<'a, 'graph>(inputs: CodecBindInputs<'a, 'graph>) -> Result<CodecBindings<'graph>, CodecBindErrors> {
    let mut bindings = BTreeMap::new();
    let mut errors = Vec::new();
    for &metadata in inputs.graph.models() {
        let model = metadata;
        for field in metadata.fields() {
            bind_field(
                model,
                field,
                inputs.graph.models(),
                inputs.codecs,
                &mut bindings,
                &mut errors,
            );
        }
        for variant in metadata.as_enum().into_iter().flat_map(|value| value.variants()) {
            for field in variant.fields() {
                let field_name = field.name().map_or_else(|| field.index().to_string(), str::to_owned);
                let path = format!("{}.{}", variant.canonical_name(), field_name);
                bind_field_at(
                    model,
                    path.into(),
                    field,
                    inputs.graph.models(),
                    inputs.codecs,
                    &mut bindings,
                    &mut errors,
                );
            }
        }
        if let Some(codec) = metadata
            .as_value()
            .and_then(crate::metadata::ValueMetadata::canonical_codec)
        {
            bind_one(
                CodecOccurrenceId::new(model, "", codec.source()),
                codec,
                metadata.type_id(),
                inputs.codecs,
                &mut bindings,
                &mut errors,
            );
        }
    }
    if errors.is_empty() {
        Ok(CodecBindings(bindings))
    } else {
        Err(CodecBindErrors::new(errors))
    }
}

/// Binds codecs declared directly on one field and its selectors.
fn bind_field<'a>(
    model: &'static TypeMetadata,
    field: &'static FieldMetadata,
    models: &[&'static TypeMetadata],
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    bind_field_at(
        model,
        field.name().unwrap_or("<unnamed>").into(),
        field,
        models,
        codecs,
        bindings,
        errors,
    );
}

/// Binds codecs for a field using the supplied canonical property path.
fn bind_field_at<'a>(
    model: &'static TypeMetadata,
    path: Box<str>,
    field: &'static FieldMetadata,
    models: &[&'static TypeMetadata],
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    let Some(descriptor) = field.descriptor() else {
        return;
    };
    let expected = descriptor
        .as_optional()
        .and_then(|value| value.element_type().concrete_type_id())
        .unwrap_or_else(|| descriptor.type_id());
    let effective = field.codec().or_else(|| canonical_codec(expected, models));
    if let Some(codec) = effective {
        bind_one(
            CodecOccurrenceId::new(model, path.clone(), codec.source()),
            codec,
            expected,
            codecs,
            bindings,
            errors,
        );
    }
    let sequence = field.sequence_constraint().and_then(|value| value.element());
    let map = field.map_constraint();
    for selector in [
        sequence,
        map.and_then(|value| value.key()),
        map.and_then(|value| value.value()),
    ]
    .into_iter()
    .flatten()
    {
        let Some(expected) = selector_type_id(descriptor, selector.position()) else {
            continue;
        };
        let Some(codec) = selector.codec().or_else(|| canonical_codec(expected, models)) else {
            continue;
        };
        bind_one(
            CodecOccurrenceId::new(model, path.clone(), CodecSource::Selector(selector.position())),
            codec,
            expected,
            codecs,
            bindings,
            errors,
        );
    }
}

/// Reads a canonical declaration only from the resolved graph's concrete
/// models.
fn canonical_codec(expected: TypeId, models: &[&'static TypeMetadata]) -> Option<&'static CodecMetadata> {
    models
        .iter()
        .find(|model| model.type_id() == expected)?
        .as_value()?
        .canonical_codec()
}

/// Resolves one declaration to exactly one compatible registry registration.
fn bind_one<'a>(
    occurrence: CodecOccurrenceId,
    declaration: &'static CodecMetadata,
    expected_type: TypeId,
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    let candidates: Vec<_> = match *declaration.codec() {
        CodecReference::DeclaredId(id) => codecs.get(id).into_iter().collect(),
        CodecReference::RustType(reference) => codecs
            .registrations()
            .iter()
            .filter(|registration| registration.descriptor().codec_type_id() == reference.type_id())
            .collect(),
    };
    let registration = match candidates.as_slice() {
        [] => {
            errors.push(CodecBindError::new(
                CodecBindErrorKind::Missing,
                occurrence,
                *declaration.codec(),
                expected_type,
                None,
                Vec::new(),
            ));
            return;
        }
        [registration] => *registration,
        _ => {
            errors.push(CodecBindError::new(
                CodecBindErrorKind::Ambiguous,
                occurrence,
                *declaration.codec(),
                expected_type,
                None,
                candidates.iter().map(|registration| registration.source()).collect(),
            ));
            return;
        }
    };
    let descriptor = registration.descriptor();
    if descriptor.value_type_id() != expected_type {
        errors.push(CodecBindError::new(
            CodecBindErrorKind::ValueTypeMismatch,
            occurrence,
            *declaration.codec(),
            expected_type,
            Some(descriptor.value_type_id()),
            vec![registration.source()],
        ));
        return;
    }
    bindings.insert(
        occurrence.clone(),
        CodecBinding {
            occurrence,
            declaration,
            descriptor,
            registration,
        },
    );
}

/// Resolves the runtime value type at one nested selector position.
fn selector_type_id(descriptor: &'static TypeDescriptor, position: SelectorPosition) -> Option<TypeId> {
    let descriptor = transparent_descriptor(descriptor)?;
    let type_ref = match position {
        SelectorPosition::Element => descriptor
            .as_sequence()
            .map(|value| value.element_type())
            .or_else(|| descriptor.as_set().map(|value| value.element_type()))
            .or_else(|| descriptor.as_array().map(|value| value.element_type()))
            .or_else(|| descriptor.as_slice().map(|value| value.element_type())),
        SelectorPosition::MapKey => descriptor.as_map().map(|value| value.key_type()),
        SelectorPosition::MapValue => descriptor.as_map().map(|value| value.value_type()),
    }?;
    type_ref.concrete_type_id()
}
