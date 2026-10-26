// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashSet;

use qubit_reflect::TypeDescriptor;

use super::abi_violation;
use super::contains_field;
use super::validate_fields;
use crate::metadata::AbiViolation;
use crate::metadata::FieldMetadata;
use crate::metadata::RoleMetadata;
use crate::type_metadata::TypeMetadata;

/// Verifies that role-specific metadata agrees with the reflected type.
///
/// # Errors
///
/// Returns an [`AbiViolation`] when role metadata has an invalid identifier,
/// transparent field, canonical codec, or enum variant definition, or when its
/// fields disagree with reflection metadata.
pub(super) fn validate_role(metadata: &TypeMetadata, descriptor: TypeDescriptor<'_>) -> Result<(), AbiViolation> {
    match metadata.role_metadata() {
        RoleMetadata::Entity(role) => validate_identifier(metadata.fields, role.identifier())?,
        RoleMetadata::Projection(role) => validate_identifier(metadata.fields, role.identifier())?,
        RoleMetadata::Model(_) => {}
        RoleMetadata::Value(role) => {
            if let Some(field) = role.transparent_field()
                && (metadata.fields.len() != 1 || !contains_field(metadata.fields, field))
            {
                return Err(abi_violation(
                    "QMM-ABI-011",
                    "a transparent value must reference its only field",
                ));
            }
            if let Some(codec) = role.canonical_codec()
                && codec.source() != crate::metadata::CodecSource::CanonicalValue
            {
                return Err(abi_violation(
                    "QMM-ABI-025",
                    "codec source differs from its metadata position",
                ));
            }
        }
        RoleMetadata::Enum(role) => {
            let reflected = descriptor.variants();
            if role.variants().len() != reflected.len() {
                return Err(abi_violation(
                    "QMM-ABI-012",
                    "enum variant metadata count differs from reflection metadata",
                ));
            }
            let mut defaults = 0;
            let mut canonical_names = HashSet::with_capacity(role.variants().len());
            let mut serialized_names = HashSet::with_capacity(role.variants().len());
            let mut deserialized_names = HashSet::with_capacity(role.variants().len());
            for (index, (variant, reflect)) in role.variants().iter().zip(reflected.iter()).enumerate() {
                if variant.index() != index
                    || reflect.index() != index
                    || reflect.declaring_type().type_id() != descriptor.type_id()
                {
                    return Err(abi_violation(
                        "QMM-ABI-012",
                        "enum variants are not in reflection source order",
                    ));
                }
                if variant.canonical_name().is_empty()
                    || !canonical_names.insert(variant.canonical_name())
                    || !serialized_names.insert(variant.serialized_name())
                    || !deserialized_names.insert(variant.deserialized_name())
                {
                    return Err(abi_violation(
                        "QMM-ABI-024",
                        "enum variant names must be non-empty and unique per namespace",
                    ));
                }
                validate_fields(variant.fields(), reflect.fields(), descriptor, "QMM-ABI-013")?;
                defaults += usize::from(variant.is_default());
            }
            if defaults > 1 {
                return Err(abi_violation(
                    "QMM-ABI-014",
                    "an enum cannot declare more than one default variant",
                ));
            }
        }
    }
    Ok(())
}

/// Verifies that an identifier belongs to the declaring field collection.
///
/// The supplied field must be declared in `fields`, marked as an identifier,
/// and be the only identifier field in that collection.
fn validate_identifier(fields: &[FieldMetadata], identifier: &FieldMetadata) -> Result<(), AbiViolation> {
    if !contains_field(fields, identifier)
        || !identifier.is_identifier()
        || fields.iter().filter(|field| field.is_identifier()).count() != 1
    {
        return Err(abi_violation(
            "QMM-ABI-010",
            "the role identifier must be an identifier field on the metadata root",
        ));
    }
    Ok(())
}
