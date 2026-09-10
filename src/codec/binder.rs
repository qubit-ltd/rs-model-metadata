// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit codec binding over an immutable model graph.
// qubit-style: allow multiple-public-types

use core::any::TypeId;
use std::collections::BTreeMap;

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistry;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;

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

/// Inputs for one codec binding pass.
pub struct CodecBindInputs<'a, 'graph> {
    /// Structure graph containing codec declarations.
    pub graph: &'graph ModelGraph<'a>,
    /// Executable codec registry.
    pub codecs: &'graph ValueCodecRegistry,
}

/// Stable identity of one codec occurrence.
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
    #[must_use]
    pub const fn model(&self) -> Option<&ModelIdBuf> {
        self.model.as_ref()
    }

    /// Returns the exact owner identity, including anonymous types.
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the normalized property path, empty for canonical value codecs.
    #[must_use]
    pub const fn property(&self) -> &str {
        &self.property
    }

    /// Returns the declaration source.
    #[must_use]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}

impl core::fmt::Display for CodecOccurrenceId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.property.is_empty() {
            write!(formatter, "{}::<canonical>", self.type_name)
        } else {
            write!(formatter, "{}::{}", self.type_name, self.property)
        }
    }
}

/// One codec declaration bound to an executable descriptor.
#[derive(Debug)]
pub struct CodecBinding<'a> {
    occurrence: CodecOccurrenceId,
    declaration: &'static CodecMetadata,
    descriptor: &'static ValueCodecDescriptor,
    registration: &'a ValueCodecRegistration,
}

impl<'a> CodecBinding<'a> {
    /// Returns the occurrence identity.
    #[must_use]
    pub const fn occurrence(&self) -> &CodecOccurrenceId {
        &self.occurrence
    }

    /// Returns the declaration metadata.
    #[must_use]
    pub const fn declaration(&self) -> &'static CodecMetadata {
        self.declaration
    }

    /// Returns the executable descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &'static ValueCodecDescriptor {
        self.descriptor
    }

    /// Returns the selected registration.
    #[must_use]
    pub const fn registration(&self) -> &'a ValueCodecRegistration {
        self.registration
    }
}

/// Immutable successful codec bindings keyed by stable occurrence identity.
#[derive(Debug)]
pub struct CodecBindings<'a>(BTreeMap<CodecOccurrenceId, CodecBinding<'a>>);

impl<'a> CodecBindings<'a> {
    /// Returns a binding by stable occurrence identity.
    #[must_use]
    pub fn get(&self, occurrence: &CodecOccurrenceId) -> Option<&CodecBinding<'a>> {
        self.0.get(occurrence)
    }

    /// Iterates over bindings in stable identity order.
    #[must_use]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = &CodecBinding<'a>> {
        self.0.values()
    }
}

/// Binds every codec declaration in a graph to an executable registration.
///
/// # Errors
///
/// Returns all missing, ambiguous, and value-type mismatched occurrences.
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
    codecs: &'a ValueCodecRegistry,
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
    codecs: &'a ValueCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    let Some(descriptor) = field.descriptor() else {
        return;
    };
    let expected = descriptor
        .as_optional()
        .and_then(|value| runtime_type_id(value.element_type()))
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
    codecs: &'a ValueCodecRegistry,
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
    runtime_type_id(type_ref)
}

/// Removes optional and smart-pointer wrappers from a resolved descriptor.
fn transparent_descriptor(mut descriptor: &'static TypeDescriptor) -> Option<&'static TypeDescriptor> {
    loop {
        let element = descriptor
            .as_optional()
            .map(|value| value.element_type())
            .or_else(|| descriptor.as_smart_pointer().map(|value| value.pointee_type()));
        let Some(element) = element else {
            return Some(descriptor);
        };
        descriptor = element.as_resolved()?;
    }
}

/// Extracts a runtime type identity from a resolved or opaque type reference.
fn runtime_type_id(type_ref: &TypeRef) -> Option<TypeId> {
    type_ref
        .as_resolved()
        .map(TypeDescriptor::type_id)
        .or_else(|| type_ref.as_opaque().map(|descriptor| descriptor.type_id()))
}
