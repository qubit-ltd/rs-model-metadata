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
mod codec_bind_report;
mod codec_bind_skip;
mod codec_bind_skip_reason;
mod codec_binding;
mod codec_bindings;
mod codec_occurrence_id;

pub use codec_bind_inputs::CodecBindInputs;
pub use codec_bind_report::CodecBindReport;
pub use codec_bind_skip::CodecBindSkip;
pub use codec_bind_skip_reason::CodecBindSkipReason;
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
    bind_core(inputs, None)
}

/// Binds codecs and reports field or selector declarations whose types could
/// not be inspected.
///
/// # Errors
///
/// Returns the same missing, ambiguous, and type-mismatch errors as
/// [`bind_codecs`]. When any such error occurs, no partial report is returned.
///
/// # Returns
///
/// Successful bindings and an ordered list of type-resolution skips. An empty
/// skip list does not prove every declaration was inspected or that every
/// field requires a codec.
///
/// # Examples
///
/// ```
/// #![cfg(feature = "codec")]
/// use std::error::Error;
///
/// use qubit_codec::ValueStringCodecRegistry;
/// use qubit_model_metadata::codec::{CodecBindInputs, bind_codecs_with_report};
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
/// let report = bind_codecs_with_report(CodecBindInputs { graph: &graph, codecs: &codecs })?;
/// assert!(report.skipped().is_empty());
/// assert_eq!(report.bindings().bindings().len(), 0);
/// # Ok(())
/// # }
/// ```
///
/// # Parameters
///
/// * `inputs` - The resolved model graph and executable codec registry.
///
/// # Returns
///
/// Successful bindings and ordered type-resolution skip diagnostics.
pub fn bind_codecs_with_report<'a, 'graph>(
    inputs: CodecBindInputs<'a, 'graph>,
) -> Result<CodecBindReport<'graph>, CodecBindErrors> {
    let mut skipped = Vec::new();
    let bindings = bind_core(inputs, Some(&mut skipped))?;
    Ok(CodecBindReport::new(bindings, skipped))
}

/// Binds codec declarations and optionally records type-resolution skips.
///
/// # Parameters
///
/// * `inputs` - The resolved model graph and executable codec registry.
/// * `skipped` - Optional destination for field and selector diagnostics.
///
/// # Returns
///
/// The successful bindings, or the aggregated binding errors.
fn bind_core<'a, 'graph>(
    inputs: CodecBindInputs<'a, 'graph>,
    mut skipped: Option<&mut Vec<CodecBindSkip>>,
) -> Result<CodecBindings<'graph>, CodecBindErrors> {
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
                skipped.as_deref_mut(),
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
                    skipped.as_deref_mut(),
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
/// * `skipped` - Optional destination for type-resolution diagnostics.
#[allow(clippy::too_many_arguments)] // The optional report sink preserves the legacy binding path.
#[inline]
fn bind_field<'a>(
    model: &'static TypeMetadata,
    field: &'static FieldMetadata,
    context: &ReflectionContext,
    graph: &ModelGraph<'_>,
    codecs: &'a ValueStringCodecRegistry,
    bindings: &mut BTreeMap<CodecOccurrenceId, CodecBinding<'a>>,
    errors: &mut Vec<CodecBindError>,
    skipped: Option<&mut Vec<CodecBindSkip>>,
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
        skipped,
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
/// * `skipped` - Optional destination for type-resolution diagnostics.
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
    mut skipped: Option<&mut Vec<CodecBindSkip>>,
) {
    let Some(descriptor) = field.reflect_in(context).and_then(|field| field.find_concrete_type()) else {
        if let Some(skipped) = skipped.as_deref_mut() {
            skipped.push(CodecBindSkip::new(
                model,
                path,
                *field.declaration(),
                CodecBindSkipReason::FieldDescriptorUnavailable,
            ));
        }
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
            if let Some(skipped) = skipped.as_deref_mut() {
                skipped.push(CodecBindSkip::new(
                    model,
                    path.clone(),
                    field.declaration().with_selector(selector.position()),
                    CodecBindSkipReason::SelectorTypeUnavailable,
                ));
            }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use qubit_codec::ValueStringCodecRegistry;
    use qubit_model_derive::Model;
    use qubit_reflect::ReflectionContext;

    use super::super::CodecBindError;
    use super::super::CodecBindSkipReason;
    use super::super::CodecBinding;
    use super::super::CodecOccurrenceId;
    use super::bind_field_at;
    use crate::metadata::ConstraintMetadata;
    use crate::metadata::FieldMetadata;
    use crate::metadata::MapConstraint;
    use crate::metadata::SelectorMetadata;
    use crate::metadata::SelectorPosition;
    use crate::metadata::SequenceConstraint;
    use crate::metadata::TypeMetadata;
    use crate::registry::ModelRegistry;
    use crate::resolve::ResolveInputs;
    use crate::resolve::StructureResolver;

    #[Model]
    struct SelectorSkipProbe {
        value: String,
    }

    static ELEMENT: SelectorMetadata = SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None);
    static MAP_KEY: SelectorMetadata = SelectorMetadata::new(SelectorPosition::MapKey, &[], &[], None, None);
    static MAP_VALUE: SelectorMetadata = SelectorMetadata::new(SelectorPosition::MapValue, &[], &[], None, None);
    static CONSTRAINTS: [ConstraintMetadata; 2] = [
        ConstraintMetadata::Sequence(SequenceConstraint::new(None, None, false).with_element(&ELEMENT)),
        ConstraintMetadata::Map(MapConstraint::new(None, None).with_selectors(Some(&MAP_KEY), Some(&MAP_VALUE))),
    ];

    #[test]
    fn test_reports_selector_skip_location_when_type_is_unavailable() {
        let root = TypeMetadata::of::<SelectorSkipProbe>();
        let reflected_field = &root.fields()[0];
        let field: &'static FieldMetadata = Box::leak(Box::new(FieldMetadata::with_semantics(
            root.type_id(),
            None,
            reflected_field.index(),
            reflected_field.name(),
            reflected_field.visibility(),
            reflected_field.type_expression_data().expect("field expression"),
            None,
            &[],
            &CONSTRAINTS,
            &[],
            reflected_field.serde(),
        )));
        let roots = [root];
        let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("isolated registry");
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("valid graph from reflected model");
        let codecs = ValueStringCodecRegistry::empty();
        let mut bindings: BTreeMap<CodecOccurrenceId, CodecBinding<'_>> = BTreeMap::new();
        let mut errors: Vec<CodecBindError> = Vec::new();
        let mut skipped = Vec::new();

        bind_field_at(
            root,
            "value".into(),
            field,
            ReflectionContext::global(),
            &graph,
            &codecs,
            &mut bindings,
            &mut errors,
            Some(&mut skipped),
        );

        assert!(bindings.is_empty());
        assert!(errors.is_empty());
        assert_eq!(skipped.len(), 3);
        let positions = [
            SelectorPosition::Element,
            SelectorPosition::MapKey,
            SelectorPosition::MapValue,
        ];
        for (skip, position) in skipped.iter().zip(positions) {
            assert_eq!(skip.model().type_id(), root.type_id());
            assert_eq!(skip.property(), "value");
            assert_eq!(skip.reason(), CodecBindSkipReason::SelectorTypeUnavailable);
            assert_eq!(skip.location().selector, Some(position));
            assert_eq!(skip.location().field, None);
        }

        // This intentionally inconsistent overlay is tested below the
        // resolver boundary: public graph construction rejects it with
        // QMM-ABI-020 before codec binding can observe it.
    }
}
