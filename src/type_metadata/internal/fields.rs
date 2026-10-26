// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::collections::HashSet;

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::Fields;
use qubit_reflect::descriptor::Reflect;
use qubit_reflect::descriptor::TypeKind;

use super::abi_violation;
use crate::metadata::AbiViolation;
use crate::metadata::ConstraintMetadata;
use crate::metadata::FieldAttributeMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorPosition;
use crate::transparent_descriptor::transparent_descriptor;

/// Rebuilds the reflection descriptor for `T` in the supplied context.
///
/// The returned descriptor carries the context needed by reflection operations
/// and should be used for the requested metadata checks.
#[must_use = "use the returned descriptor for reflection operations"]
pub(in crate::type_metadata) fn descriptor_for<'ctx, T: Reflect + 'static>(
    context: &'ctx ReflectionContext,
) -> TypeDescriptor<'ctx> {
    TypeDescriptor::of_in::<T>(context)
}

/// Verifies that field overlays mirror the reflection descriptor exactly.
pub(in crate::type_metadata) fn validate_fields(
    metadata: &[FieldMetadata],
    reflected: Fields<'_>,
    descriptor: TypeDescriptor<'_>,
    code: &'static str,
) -> Result<(), AbiViolation> {
    let reflected_count = metadata.iter().filter(|field| !field.is_opaque()).count();
    if reflected_count != reflected.len() {
        return Err(abi_violation(
            code,
            "field metadata count differs from reflection metadata",
        ));
    }
    for (index, field) in metadata.iter().enumerate() {
        let Some(location) = field.location() else {
            return Err(abi_violation(
                code,
                "concrete field metadata has no declaration identity",
            ));
        };
        if location.owner() != descriptor.type_id() || location.index() != index || field.index() != index {
            return Err(abi_violation(code, "field metadata identity differs from source order"));
        }
        if field.is_opaque() {
            validate_field_semantics(field, descriptor.context())?;
            continue;
        }
        let Some(reflect) = reflected.iter().find(|candidate| candidate.index() == field.index()) else {
            return Err(abi_violation(
                code,
                "visible field metadata has no reflection field at its source index",
            ));
        };
        if location.variant() != reflect.declaring_variant().map(|variant| variant.index())
            || !core::ptr::eq(reflect.context(), descriptor.context())
            || reflect.declaring_type().type_id() != descriptor.type_id()
        {
            return Err(abi_violation(
                code,
                "field metadata identity differs from reflection source order",
            ));
        }
        validate_field_semantics(field, descriptor.context())?;
    }
    validate_key_part_order(metadata)?;
    Ok(())
}

/// Verifies that key-part orders are unique and contiguous.
fn validate_key_part_order(metadata: &[FieldMetadata]) -> Result<(), AbiViolation> {
    let mut key_parts = metadata
        .iter()
        .filter_map(|field| field.key_part().map(|key_part| key_part.order()))
        .collect::<Vec<_>>();
    key_parts.sort_unstable();
    if key_parts.iter().copied().ne(0..key_parts.len()) {
        return Err(abi_violation(
            "QMM-ABI-019",
            "key-part orders must be unique and contiguous",
        ));
    }
    Ok(())
}

/// Verifies mutually compatible field-level semantic declarations.
fn validate_field_semantics(field: &FieldMetadata, context: &ReflectionContext) -> Result<(), AbiViolation> {
    // Every counter is bounded by the slice length, so usize cannot overflow
    // even for malformed generated input containing more than 255 repeats.
    let mut singleton_counts = [0_usize; 9];
    let mut constraints = Vec::new();
    let mut validators = Vec::new();
    for attribute in field.attributes() {
        match attribute {
            FieldAttributeMetadata::Identifier(_) => singleton_counts[0] += 1,
            FieldAttributeMetadata::Unique(_) => singleton_counts[1] += 1,
            FieldAttributeMetadata::Reference(_) => singleton_counts[2] += 1,
            FieldAttributeMetadata::KeyPart(_) => singleton_counts[3] += 1,
            FieldAttributeMetadata::Codec(_) => singleton_counts[4] += 1,
            FieldAttributeMetadata::Redact(_) => singleton_counts[5] += 1,
            FieldAttributeMetadata::Serde(value) => {
                singleton_counts[6] += 1;
                if !core::ptr::eq(*value, field.serde()) {
                    return Err(abi_violation(
                        "QMM-ABI-017",
                        "serde occurrence does not match the cached field value",
                    ));
                }
            }
            FieldAttributeMetadata::Opaque => singleton_counts[7] += 1,
            FieldAttributeMetadata::Indexed(_) => singleton_counts[8] += 1,
            FieldAttributeMetadata::Constraint(value) => constraints.push(*value),
            FieldAttributeMetadata::Validator(value) => validators.push(*value),
        }
    }
    if singleton_counts.into_iter().any(|count| count > 1) {
        return Err(abi_violation(
            "QMM-ABI-015",
            "a field contains duplicate singleton semantics",
        ));
    }
    if constraints.len() != field.constraints().len()
        || !constraints
            .iter()
            .zip(field.constraints())
            .all(|(left, right)| core::ptr::eq(*left, right))
        || validators.len() != field.validators().len()
        || !validators
            .iter()
            .zip(field.validators())
            .all(|(left, right)| core::ptr::eq(*left, right))
    {
        return Err(abi_violation(
            "QMM-ABI-016",
            "field occurrence lists and cached slices disagree",
        ));
    }
    let concrete_type = field
        .reflect_in(context)
        .and_then(|reflected| reflected.find_concrete_type());
    validate_constraint_kinds(field.constraints(), concrete_type, true)?;
    validate_validators(field.validators())?;
    validate_codec(
        field.codec(),
        concrete_type.map(|descriptor| descriptor.type_id()),
        crate::metadata::CodecSource::Field,
    )?;
    Ok(())
}

/// Verifies that validator declarations have non-empty, unique dependencies.
fn validate_validators(validators: &[crate::metadata::ValidatorMetadata]) -> Result<(), AbiViolation> {
    for validator in validators {
        let mut parameter_names = HashSet::with_capacity(validator.params().len());
        if validator
            .params()
            .iter()
            .any(|argument| !parameter_names.insert(argument.name()))
        {
            return Err(abi_violation("QMM-ABI-018", "validator parameter names must be unique"));
        }
        for (index, dependency) in validator.depends_on().iter().enumerate() {
            if dependency.is_empty() || validator.depends_on()[..index].contains(dependency) {
                return Err(abi_violation(
                    "QMM-ABI-018",
                    "validator dependency paths must be non-empty and unique",
                ));
            }
        }
    }
    Ok(())
}

/// Verifies constraint kinds against the reflected field type.
fn validate_constraint_kinds(
    constraints: &[ConstraintMetadata],
    descriptor: Option<TypeDescriptor<'_>>,
    allow_selectors: bool,
) -> Result<(), AbiViolation> {
    let mut kinds = HashSet::with_capacity(constraints.len());
    for constraint in constraints {
        let kind = match constraint {
            ConstraintMetadata::Text(_) => 0,
            ConstraintMetadata::Decimal(_) => 1,
            ConstraintMetadata::Time(_) => 2,
            ConstraintMetadata::Sequence(sequence) => {
                if let Some(selector) = sequence.element() {
                    if !allow_selectors {
                        return Err(abi_violation("QMM-ABI-020", "selector semantics must be non-recursive"));
                    }
                    validate_selector(selector, SelectorPosition::Element, descriptor)?;
                }
                3
            }
            ConstraintMetadata::Map(map) => {
                for (selector, position) in [
                    (map.key(), SelectorPosition::MapKey),
                    (map.value(), SelectorPosition::MapValue),
                ] {
                    if let Some(selector) = selector {
                        if !allow_selectors {
                            return Err(abi_violation("QMM-ABI-020", "selector semantics must be non-recursive"));
                        }
                        validate_selector(selector, position, descriptor)?;
                    }
                }
                4
            }
        };
        if !kinds.insert(kind) {
            return Err(abi_violation(
                "QMM-ABI-021",
                "a field contains duplicate constraint kinds",
            ));
        }
    }
    Ok(())
}

/// Verifies one nested selector against its structural position and type.
fn validate_selector(
    selector: &crate::metadata::SelectorMetadata,
    position: SelectorPosition,
    descriptor: Option<TypeDescriptor<'_>>,
) -> Result<(), AbiViolation> {
    if selector.position() != position {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector has the wrong structural position",
        ));
    }
    let Some(selected_type) = descriptor
        .and_then(transparent_descriptor)
        .and_then(|descriptor| selector_type_descriptor(descriptor, position))
    else {
        return Err(abi_violation(
            "QMM-ABI-020",
            "selector position is incompatible with the field type",
        ));
    };
    validate_constraint_kinds(selector.constraints(), Some(selected_type), false)?;
    validate_validators(selector.validators())?;
    validate_codec(
        selector.codec(),
        Some(selected_type.type_id()),
        crate::metadata::CodecSource::Selector(position),
    )
}

/// Verifies that a codec declaration is valid for its expected type and source.
fn validate_codec(
    codec: Option<&crate::metadata::CodecMetadata>,
    expected_type: Option<TypeId>,
    source: crate::metadata::CodecSource,
) -> Result<(), AbiViolation> {
    let Some(codec) = codec else {
        return Ok(());
    };
    if codec.source() != source {
        return Err(abi_violation(
            "QMM-ABI-025",
            "codec source differs from its metadata position",
        ));
    }
    let _ = expected_type;
    Ok(())
}

/// Returns the nested type selected by a collection position.
fn selector_type_descriptor(descriptor: TypeDescriptor<'_>, position: SelectorPosition) -> Option<TypeDescriptor<'_>> {
    let arguments = descriptor.generic_arguments();
    let index = match (descriptor.kind(), position) {
        (
            TypeKind::Sequence(_) | TypeKind::Set(_) | TypeKind::Array(_) | TypeKind::Slice(_),
            SelectorPosition::Element,
        ) => 0,
        (TypeKind::Map(_), SelectorPosition::MapKey) => 0,
        (TypeKind::Map(_), SelectorPosition::MapValue) => 1,
        _ => return None,
    };
    arguments.get_at(index)?.type_argument().ok()?.find_concrete_type()
}
