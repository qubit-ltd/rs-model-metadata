// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One validated property access retained for a later executor.

use std::any::TypeId;

use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;

/// One validated property access retained for a later executor.
///
/// The metadata is copied from the resolved property set or constructed from
/// a transparent value field. Inspecting a step never invokes a getter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    /// Validated metadata copied for the property at this path segment.
    pub(super) property: PropertyMetadata,
    /// Model metadata that owns the property, or `None` for a synthetic step.
    pub(super) owner: Option<&'static TypeMetadata>,
    /// Exact element type expected after projecting a borrowed `Option<T>`.
    pub(super) optional_element: Option<TypeId>,
}

impl PropertyStep {
    /// Returns the metadata for this path segment.
    ///
    /// The returned metadata is a small copy of the validated descriptor and
    /// can be inspected without executing user code.
    ///
    /// # Returns
    ///
    /// The copy of the property metadata retained for this path segment.
    #[must_use]
    #[inline]
    pub(crate) const fn property(self) -> PropertyMetadata {
        self.property
    }

    /// Returns the metadata owner, when this step was compiled from a model.
    ///
    /// # Returns
    ///
    /// The model metadata that owns the property, or `None` for synthetic
    /// steps.
    #[inline]
    pub(crate) const fn owner(self) -> Option<&'static TypeMetadata> {
        self.owner
    }

    /// Returns the optional element type to project for intermediate traversal.
    ///
    /// # Returns
    ///
    /// The reflected `TypeId` expected from the optional value, or `None` when
    /// this step does not project an optional element.
    #[inline]
    pub(crate) const fn optional_element(self) -> Option<TypeId> {
        self.optional_element
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;
    use qubit_validator::ExecutionErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::ValidationPath;

    use super::PropertyStep;
    use crate::__private::v8;
    use crate::field_metadata::FieldMetadata;
    use crate::metadata::PropertyMetadata;
    use crate::validation::ValidationOptions;
    use crate::validation::compiled_property_path::CompiledPropertyPath;
    use crate::validation::internal::execution_budget::ExecutionBudget;
    use crate::validation::internal::path_reader;

    #[Reflect]
    struct OptionalField {
        value: Option<String>,
    }

    fn path(property: PropertyMetadata, element: Option<TypeId>) -> CompiledPropertyPath {
        CompiledPropertyPath {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: vec![
                PropertyStep {
                    property,
                    owner: None,
                    optional_element: element,
                },
                PropertyStep {
                    property,
                    owner: None,
                    optional_element: None,
                },
            ]
            .into_boxed_slice(),
            optional_steps: Box::new([]),
            input: InputType::of::<String>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        }
    }

    fn property(context: &ReflectionContext, optional_element: Option<TypeId>) -> PropertyMetadata {
        let descriptor = TypeDescriptor::of_in::<OptionalField>(context);
        let mut field = FieldMetadata::from_reflect(
            descriptor.type_id(),
            descriptor.fields().get_at(0).expect("optional field"),
        );
        if let Some(element) = optional_element {
            field = if element == TypeId::of::<String>() {
                field.with_optional_ops(crate::property::OptionalOps::for_element::<String>())
            } else {
                field.with_optional_ops(crate::property::OptionalOps::for_element::<u32>())
            };
        }
        *v8::leak(PropertyMetadata::new(
            "value",
            field.type_expression_data(),
            Some(v8::leak(field)),
            None,
            None,
        ))
    }

    #[test]
    fn test_read_rejects_unavailable_and_mismatched_optional_projection_adapters() {
        let context = ReflectionContext::global();
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let root = OptionalField {
            value: Some("value".to_owned()),
        };

        let missing_ops = path(property(context, None), Some(TypeId::of::<String>()));
        let error = match path_reader::read(
            context,
            &missing_ops,
            crate::metadata::PropertyBorrowed::new(&root),
            0,
            &mut budget,
        ) {
            Ok(_) => panic!("optional projection without adapter unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert_eq!(
            error.path(),
            &ValidationPath::root().with_field("value").with_field("value")
        );

        let wrong_ops = path(
            property(context, Some(TypeId::of::<u32>())),
            Some(TypeId::of::<String>()),
        );
        let error = match path_reader::read(
            context,
            &wrong_ops,
            crate::metadata::PropertyBorrowed::new(&root),
            0,
            &mut budget,
        ) {
            Ok(_) => panic!("mismatched optional adapter unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert!(error.trusted_source().is_some());

        let unexpected_element = path(
            property(context, Some(TypeId::of::<String>())),
            Some(TypeId::of::<u32>()),
        );
        let error = match path_reader::read(
            context,
            &unexpected_element,
            crate::metadata::PropertyBorrowed::new(&root),
            0,
            &mut budget,
        ) {
            Ok(_) => panic!("unexpected optional element type succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
    }
}
