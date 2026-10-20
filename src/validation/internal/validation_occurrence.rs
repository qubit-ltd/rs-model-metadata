// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Context of one executable declaration at a concrete usage path.

use std::fmt::Debug;
use std::fmt::Formatter;

use super::execution_declaration::ExecutionDeclaration;
use crate::metadata::EnumVariantMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;

/// One source declaration reached through one root-relative field path.
#[derive(Clone)]
pub(crate) struct ValidationOccurrence {
    /// Root requested by the caller.
    pub(crate) root: &'static TypeMetadata,
    /// Concrete type declaring the field.
    pub(crate) owner: &'static TypeMetadata,
    /// Original field overlay, including its checked identity and source.
    pub(crate) field: &'static FieldMetadata,
    /// Enum variant containing this payload field, when the owner is an enum.
    pub(crate) enum_variant: Option<&'static EnumVariantMetadata>,
    /// Full property path from the root to this field.
    pub(crate) segments: Vec<&'static str>,
    /// Structural path for rejected shapes, including variant and tuple
    /// positions. Supported property paths use `segments` without
    /// allocating this string.
    pub(crate) unsupported_path: Option<String>,
    /// Container position, if this declaration belongs to a selector.
    pub(crate) selector: Option<SelectorPosition>,
    /// Declaration-order identity within the root's traversal.
    pub(crate) ordinal: usize,
    /// Original executable declaration.
    pub(crate) declaration: ExecutionDeclaration,
}

impl ValidationOccurrence {
    /// Returns the declared user rule ID even when no registration exists.
    ///
    /// # Returns
    /// The declared ID for validator occurrences, or `None` for constraints
    /// and traversal-only occurrences.
    #[inline]
    #[must_use]
    pub(crate) const fn declared_rule_id(&self) -> Option<&'static str> {
        match self.declaration {
            ExecutionDeclaration::Validator(value) => Some(value.declared_id()),
            ExecutionDeclaration::Constraint(_) | ExecutionDeclaration::Traversal => None,
        }
    }
}

impl Debug for ValidationOccurrence {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidationOccurrence")
            .field("root", &self.root.type_name())
            .field("owner", &self.owner.type_name())
            .field("field", &self.field.location())
            .field("enum_variant", &self.enum_variant.map(|variant| variant.rust_name()))
            .field("selector", &self.selector)
            .field("ordinal", &self.ordinal)
            .field("declared_rule_id", &self.declared_rule_id())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;

    use super::ExecutionDeclaration;
    use super::ValidationOccurrence;
    use crate::__private::v8;
    use crate::metadata::EnumVariantMetadata;
    use crate::metadata::FieldMetadata;
    use crate::metadata::TypeMetadata;
    use crate::metadata::ValidatorMetadata;

    #[Reflect]
    struct OccurrenceFixture {
        _value: String,
    }

    #[Reflect]
    enum VariantFixture {
        Ready,
    }

    static VALIDATOR: ValidatorMetadata = ValidatorMetadata::new("example.rule", &[], &[]);

    fn fixture_metadata() -> &'static TypeMetadata {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<OccurrenceFixture>(context);
        let fields = v8::leak_slice(vec![FieldMetadata::from_reflect(
            descriptor.type_id(),
            descriptor.fields().get_at(0).expect("fixture field descriptor"),
        )]);
        v8::leak(TypeMetadata::new::<OccurrenceFixture>(
            None,
            fields,
            v8::leak(v8::model_role()),
        ))
    }

    fn fixture_variant() -> &'static EnumVariantMetadata {
        let descriptor = TypeDescriptor::of_in::<VariantFixture>(ReflectionContext::global());
        let reflect = descriptor.variants().get_at(0).expect("fixture enum variant");
        v8::leak(v8::enum_variant_metadata(
            descriptor.type_id(),
            reflect.index(),
            "Ready",
            "READY",
            "READY",
            "READY",
            &[],
            false,
        ))
    }

    #[test]
    fn test_validator_occurrence_rule_id_and_debug_format() {
        let metadata = fixture_metadata();
        let occurrence = ValidationOccurrence {
            root: metadata,
            owner: metadata,
            field: metadata.fields().first().expect("fixture field metadata"),
            enum_variant: Some(fixture_variant()),
            segments: vec!["_value"],
            unsupported_path: None,
            selector: None,
            ordinal: 0,
            declaration: ExecutionDeclaration::Validator(&VALIDATOR),
        };

        assert_eq!(occurrence.declared_rule_id(), Some("example.rule"));
        let debug = format!("{occurrence:?}");
        assert!(debug.contains("OccurrenceFixture"));
        assert!(debug.contains("example.rule"));
        assert!(debug.contains("Ready"));
    }
}
