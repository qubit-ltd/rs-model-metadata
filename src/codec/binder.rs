// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit codec binding over an immutable model graph.

use core::any::TypeId;
use std::collections::BTreeMap;

use qubit_codec::ValueStringCodecRegistry;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;

use super::CodecBindError;
use super::CodecBindErrorKind;
use super::CodecBindErrors;
use crate::metadata::CodecMetadata;
use crate::metadata::CodecReference;
use crate::metadata::CodecSource;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::transparent_descriptor::transparent_descriptor;

mod codec_bind_inputs;
mod codec_binding;
mod codec_bindings;
mod codec_occurrence_id;

pub use codec_bind_inputs::CodecBindInputs;
pub use codec_binding::CodecBinding;
pub use codec_bindings::CodecBindings;
pub use codec_occurrence_id::CodecOccurrenceId;

/// Binds every codec declaration in a graph to an executable registration.
///
/// For a direct field declaration or canonical value fallback, the required
/// value type is the field type itself, except that one outer `Option<T>` is
/// unwrapped to `T`. Smart pointers are matched exactly (`Box<T>` requires a
/// `Box<T>` codec), and nested options are not recursively unwrapped. A
/// canonical codec for `T` is therefore inherited by `T` and `Option<T>`
/// fields, but not by `Box<T>`, `Option<Box<T>>`, or `Option<Option<T>>`.
/// Collection selector declarations use their separate selector type rules.
/// A field is processed only when its concrete descriptor can be resolved in
/// the graph's reflection context. If that descriptor is unavailable, the
/// field and all of its codec declarations are skipped. Consequently, success
/// means that every processed declaration bound without error; it does not
/// guarantee that every source declaration was checked or produced a binding.
///
/// This function resolves declarations and validates registration types. It
/// does not call the selected codec's `encode` or `decode` methods; consumers
/// execute the returned descriptors with values of the registered type.
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
/// All successful bindings, keyed by process-local codec occurrence identity.
pub fn bind_codecs<'a, 'graph>(inputs: CodecBindInputs<'a, 'graph>) -> Result<CodecBindings<'graph>, CodecBindErrors> {
    let mut bindings = BTreeMap::new();
    let mut errors = Vec::new();
    let context = inputs
        .graph
        .registry()
        .context()
        .expect("resolved model graphs retain their reflection context");
    for &metadata in inputs.graph.models() {
        let model = metadata;
        for field in metadata.fields() {
            bind_field(
                model,
                field,
                context,
                inputs.graph,
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
                    context,
                    inputs.graph,
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
///
/// # Parameters
///
/// * `model` - Metadata for the model that owns the field.
/// * `field` - Field whose direct and selector codec declarations are bound.
/// * `context` - Reflection context used to inspect the field's concrete type.
/// * `graph` - Resolved graph used to find canonical value codecs.
/// * `codecs` - Runtime registry of executable value-string codecs.
/// * `bindings` - Output map receiving successful bindings by occurrence.
/// * `errors` - Output list receiving binding failures.
#[inline]
fn bind_field<'a>(
    model: &'static TypeMetadata,
    field: &'static FieldMetadata,
    context: &ReflectionContext,
    graph: &ModelGraph<'_>,
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    bind_field_at(
        model,
        field.name().unwrap_or("<unnamed>").into(),
        field,
        context,
        graph,
        codecs,
        bindings,
        errors,
    );
}

/// Binds codecs for a field using the supplied canonical property path.
///
/// # Parameters
///
/// * `model` - Metadata for the model that owns the field.
/// * `path` - Canonical property path used to identify codec occurrences.
/// * `field` - Field whose direct and selector codec declarations are bound.
/// * `context` - Reflection context used to inspect the field's concrete type.
/// * `graph` - Resolved graph used to find canonical value codecs.
/// * `codecs` - Runtime registry of executable value-string codecs.
/// * `bindings` - Output map receiving successful bindings by occurrence.
/// * `errors` - Output list receiving binding failures.
#[allow(clippy::too_many_arguments)]
fn bind_field_at<'a>(
    model: &'static TypeMetadata,
    path: Box<str>,
    field: &'static FieldMetadata,
    context: &ReflectionContext,
    graph: &ModelGraph<'_>,
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
) {
    let Some(descriptor) = field.reflect_in(context).and_then(|field| field.find_concrete_type()) else {
        return;
    };
    let expected = match descriptor.kind() {
        TypeKind::Optional(_) => descriptor
            .generic_arguments()
            .get_at(0)
            .and_then(|argument| argument.type_argument().ok())
            .and_then(|argument| argument.find_concrete_type())
            .map(TypeDescriptor::type_id)
            .unwrap_or_else(|| descriptor.type_id()),
        _ => descriptor.type_id(),
    };
    let effective = field.codec().or_else(|| canonical_codec(expected, graph));
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
        let Some(codec) = selector.codec().or_else(|| canonical_codec(expected, graph)) else {
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
///
/// # Parameters
///
/// * `expected` - Runtime type whose canonical codec is being looked up.
/// * `graph` - Resolved graph whose model index is queried for that type.
fn canonical_codec(expected: TypeId, graph: &ModelGraph<'_>) -> Option<&'static CodecMetadata> {
    graph.model(expected)?.as_value()?.canonical_codec()
}

/// Resolves one declaration to exactly one compatible registry registration.
///
/// # Parameters
///
/// * `occurrence` - Process-local identity of the codec declaration being
///   resolved.
/// * `declaration` - Codec metadata declaring the requested registration.
/// * `expected_type` - Runtime value type that the registration must support.
/// * `codecs` - Runtime registry searched for compatible registrations.
/// * `bindings` - Output map receiving the unique compatible binding.
/// * `errors` - Output list receiving missing, ambiguous, or mismatch failures.
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
///
/// # Parameters
///
/// * `descriptor` - Concrete descriptor for the selected field value.
/// * `position` - Sequence element, map key, or map value position to inspect.
#[inline]
fn selector_type_id(descriptor: TypeDescriptor<'_>, position: SelectorPosition) -> Option<TypeId> {
    let descriptor = transparent_descriptor(descriptor)?;
    let argument_index = match (descriptor.kind(), position) {
        (
            TypeKind::Sequence(_) | TypeKind::Set(_) | TypeKind::Array(_) | TypeKind::Slice(_),
            SelectorPosition::Element,
        ) => 0,
        (TypeKind::Map(_), SelectorPosition::MapKey) => 0,
        (TypeKind::Map(_), SelectorPosition::MapValue) => 1,
        _ => return None,
    };
    descriptor
        .generic_arguments()
        .get_at(argument_index)?
        .type_argument()
        .ok()?
        .find_concrete_type()
        .map(TypeDescriptor::type_id)
}
