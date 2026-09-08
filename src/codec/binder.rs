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
    model: ModelIdBuf,
    property: Box<str>,
    source: CodecSource,
}

impl CodecOccurrenceId {
    /// Creates a stable occurrence identity.
    #[must_use]
    pub fn new(model: ModelIdBuf, property: impl Into<Box<str>>, source: CodecSource) -> Self {
        Self {
            model,
            property: property.into(),
            source,
        }
    }

    /// Returns the owning model ID.
    #[must_use]
    pub const fn model(&self) -> &ModelIdBuf {
        &self.model
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
            write!(formatter, "{}::<canonical>", self.model)
        } else {
            write!(formatter, "{}::{}", self.model, self.property)
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
    for (metadata, _) in inputs.graph.registry().concrete_entries() {
        let model = ModelIdBuf::from(metadata.model_id().expect("registered concrete models have stable IDs"));
        for field in metadata.fields() {
            bind_field(model.clone(), field, inputs.codecs, &mut bindings, &mut errors);
        }
        for variant in metadata.as_enum().into_iter().flat_map(|value| value.variants()) {
            for field in variant.fields() {
                let path = format!("{}.{}", variant.canonical_name(), field.name().unwrap_or("<unnamed>"));
                bind_field_at(
                    model.clone(),
                    path.into(),
                    field,
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

fn bind_field<'a>(
    model: ModelIdBuf,
    field: &'static FieldMetadata,
    codecs: &'a ValueCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    bind_field_at(
        model,
        field.name().unwrap_or("<unnamed>").into(),
        field,
        codecs,
        bindings,
        errors,
    );
}

fn bind_field_at<'a>(
    model: ModelIdBuf,
    path: Box<str>,
    field: &'static FieldMetadata,
    codecs: &'a ValueCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    let Some(descriptor) = field.descriptor() else {
        return;
    };
    if let Some(codec) = field.codec() {
        let expected = descriptor
            .as_optional()
            .and_then(|value| runtime_type_id(value.element_type()))
            .unwrap_or_else(|| descriptor.type_id());
        bind_one(
            CodecOccurrenceId::new(model.clone(), path.clone(), codec.source()),
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
        let Some(codec) = selector.codec() else {
            continue;
        };
        let Some(expected) = selector_type_id(descriptor, selector.position()) else {
            continue;
        };
        bind_one(
            CodecOccurrenceId::new(model.clone(), path.clone(), codec.source()),
            codec,
            expected,
            codecs,
            bindings,
            errors,
        );
    }
}

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

fn runtime_type_id(type_ref: &TypeRef) -> Option<TypeId> {
    type_ref
        .as_resolved()
        .map(TypeDescriptor::type_id)
        .or_else(|| type_ref.as_opaque().map(|descriptor| descriptor.type_id()))
}
