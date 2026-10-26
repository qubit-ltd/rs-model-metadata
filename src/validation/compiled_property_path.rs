// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural, getter-free compilation of property paths.

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::GenericArgumentDescriptor;
use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;

use super::build_error::path_error;
use crate::metadata::GetterOutputKind;
use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;

mod compiled_property_path;
mod property_step;
mod variant_guard;

pub(crate) use compiled_property_path::CompiledPropertyPath;
pub(crate) use property_step::PropertyStep;
pub(crate) use variant_guard::VariantGuard;

/// Resolves a property's getter shape and validates optional intermediate
/// reads.
///
/// # Parameters
///
/// The arguments describe the current property, its declared descriptor, and
/// whether this path segment is terminal or value-targeted.
///
/// # Returns
///
/// The actual reflected output, whether it is optional, and whether execution
/// must project a borrowed optional element.
///
/// # Errors
///
/// Returns `UnsupportedInput` for an unresolved getter output and
/// `UnsupportedConstraint` for an unsupported getter or optional shape.
fn resolve_getter_output<'ctx>(
    context: &'ctx ReflectionContext,
    current: &'static TypeMetadata,
    property: PropertyMetadata,
    descriptor: TypeDescriptor<'ctx>,
    expected: TypeDescriptor<'ctx>,
    last: bool,
    value_target: bool,
    declared_optional: bool,
) -> Result<(TypeDescriptor<'ctx>, bool, bool), BindError> {
    if let Some(getter) = property.getter() {
        let output = context
            .find_type_by_id(getter.output_type_id())
            .ok()
            .flatten()
            .or_else(|| {
                (getter.output_kind() != GetterOutputKind::Owned)
                    .then(|| property.descriptor_in_owner(current, context).ok().flatten())
                    .flatten()
            })
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
        if !last
            && matches!(
                getter.output_kind(),
                GetterOutputKind::Owned | GetterOutputKind::BorrowedSlice
            )
        {
            return Err(path_error(BindErrorKind::UnsupportedConstraint));
        }
        if matches!(getter.output_kind(), GetterOutputKind::OptionalBorrowed) {
            let inner =
                generic_type_argument(output, 0).ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
            Ok((inner, true, false))
        } else if last && !value_target && getter.output_kind() == GetterOutputKind::BorrowedSlice {
            Ok((expected, false, false))
        } else if !last && declared_optional {
            if getter.output_kind() != GetterOutputKind::Borrowed {
                return Err(path_error(BindErrorKind::UnsupportedConstraint));
            }
            let actual_optional =
                generic_type_argument(output, 0).ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
            if property
                .optional_ops()
                .is_none_or(|ops| ops.element_type_id() != expected.type_id())
                || actual_optional.type_id() != expected.type_id()
            {
                return Err(path_error(BindErrorKind::UnsupportedConstraint));
            }
            Ok((actual_optional, true, true))
        } else {
            Ok((output, false, false))
        }
    } else if declared_optional {
        if property
            .optional_ops()
            .is_none_or(|ops| ops.element_type_id() != expected.type_id())
        {
            return Err(path_error(BindErrorKind::UnsupportedConstraint));
        }
        Ok((expected, true, true))
    } else {
        Ok((descriptor, false, false))
    }
}

/// Removes transparent wrappers and records whether an optional was found.
///
/// # Parameters
///
/// - `descriptor`: reflected property type to inspect.
///
/// # Returns
///
/// The innermost resolved descriptor and whether an optional wrapper was
/// encountered. If a wrapper cannot be resolved, returns the current
/// descriptor and the optionality observed so far.
fn value_descriptor<'ctx>(mut descriptor: TypeDescriptor<'ctx>) -> (TypeDescriptor<'ctx>, bool) {
    let mut optional = false;
    loop {
        let kind = descriptor.kind();
        let is_optional = matches!(kind, TypeKind::Optional(_));
        let is_smart_pointer = matches!(kind, TypeKind::SmartPointer(_));
        if !is_optional && !is_smart_pointer {
            return (descriptor, optional);
        }
        optional |= is_optional;
        let Some(next) = generic_type_argument(descriptor, 0) else {
            return (descriptor, optional);
        };
        descriptor = next;
    }
}

/// Resolves one generic argument to its concrete reflected type, when
/// available.
///
/// # Parameters
///
/// - `descriptor`: reflected generic type whose arguments are inspected.
/// - `index`: zero-based position of the requested generic argument.
///
/// # Returns
///
/// The concrete argument descriptor, or `None` when the index is out of range,
/// the type expression cannot be resolved, or it is not concrete.
fn generic_type_argument<'ctx>(descriptor: TypeDescriptor<'ctx>, index: usize) -> Option<TypeDescriptor<'ctx>> {
    let argument = descriptor.generic_arguments().iter().nth(index)?;
    concrete_generic_type_argument(argument)
}

/// Resolves a selected generic argument when it names a concrete type.
///
/// # Parameters
///
/// - `argument`: selected reflected generic argument.
///
/// # Returns
///
/// The concrete type descriptor, or `None` for a const argument, unresolved
/// type expression, or unavailable descriptor.
fn concrete_generic_type_argument<'ctx>(argument: GenericArgumentDescriptor<'ctx>) -> Option<TypeDescriptor<'ctx>> {
    argument.type_argument().ok()?.find_concrete_type()
}

#[cfg(test)]
mod tests {
    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;
    use qubit_validator::InputType;

    use super::CompiledPropertyPath;
    use super::PropertyStep;
    use super::generic_type_argument;
    use super::value_descriptor;
    use crate::__private::v8;
    use crate::field_metadata::FieldMetadata;
    use crate::property::OptionalOps;
    use crate::property::PropertyMetadata;

    #[Reflect]
    struct OptionalTerminalFixture {
        value: Option<String>,
    }

    #[test]
    fn test_compiled_path_accessors_expose_structural_state() {
        let path = CompiledPropertyPath {
            context_depth: 2,
            dependency: None,
            deferred: vec!["parent", "value"].into_boxed_slice(),
            steps: Box::new([]),
            optional_steps: Box::new([]),
            input: InputType::of::<u8>(),
            optional: true,
            variant_guard: None,
            variant_name: None,
        };

        assert!(path.dependency().is_none());
        assert_eq!(path.deferred(), &["parent", "value"]);
        assert_eq!(path.context_depth(), 2);
        assert!(path.steps().is_empty());
        assert_eq!(path.input_type(), InputType::of::<u8>());
        assert!(path.is_optional());
    }

    #[test]
    fn test_optional_terminal_projection_and_prefix_coverage() {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<OptionalTerminalFixture>(context);
        let field = v8::leak(
            FieldMetadata::from_reflect(
                descriptor.type_id(),
                descriptor.fields().get_at(0).expect("optional terminal field"),
            )
            .with_optional_ops(OptionalOps::for_element::<String>()),
        );
        let property = PropertyMetadata::new("value", field.type_expression_data(), Some(field), None, None);
        let path = CompiledPropertyPath {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: vec![PropertyStep {
                property,
                owner: None,
                optional_element: None,
            }]
            .into_boxed_slice(),
            optional_steps: vec![0].into_boxed_slice(),
            input: InputType::of::<Option<String>>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        }
        .unwrap_terminal_optional(
            generic_type_argument(descriptor.fields().get_at(0).unwrap().find_concrete_type().unwrap(), 0).unwrap(),
        )
        .expect("Option<String> projects to its reflected text element");

        assert_eq!(path.input_type(), InputType::Text);
        assert!(path.is_optional());

        let covered_target = CompiledPropertyPath {
            optional: true,
            ..path.clone()
        };
        assert!(path.optionality_covered_by(&covered_target, context));

        let uncovered_target = CompiledPropertyPath {
            optional_steps: Box::new([]),
            ..covered_target
        };
        assert!(!path.optionality_covered_by(&uncovered_target, context));

        let (element, optional) =
            value_descriptor(descriptor.fields().get_at(0).unwrap().find_concrete_type().unwrap());
        assert_eq!(element.type_id(), TypeDescriptor::of_in::<String>(context).type_id());
        assert!(optional);
    }

    #[test]
    fn test_optional_terminal_projection_rejects_an_unrelated_element_type() {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<OptionalTerminalFixture>(context);
        let field = v8::leak(
            FieldMetadata::from_reflect(
                descriptor.type_id(),
                descriptor.fields().get_at(0).expect("optional terminal field"),
            )
            .with_optional_ops(OptionalOps::for_element::<String>()),
        );
        let path = CompiledPropertyPath {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: vec![PropertyStep {
                property: PropertyMetadata::new("value", field.type_expression_data(), Some(field), None, None),
                owner: None,
                optional_element: None,
            }]
            .into_boxed_slice(),
            optional_steps: vec![0].into_boxed_slice(),
            input: InputType::of::<Option<String>>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        };

        let error = path
            .unwrap_terminal_optional(TypeDescriptor::of_in::<u64>(context))
            .expect_err("Option<String> cannot project to an unrelated element type");
        assert_eq!(error.kind(), qubit_validator::BindErrorKind::UnsupportedConstraint);
    }

    #[test]
    fn test_value_descriptor_unwraps_optional_and_smart_pointer_layers() {
        let context = ReflectionContext::global();
        let optional = TypeDescriptor::of_in::<Option<String>>(context);
        let (value, was_optional) = value_descriptor(optional);
        assert!(was_optional);
        assert_eq!(value.type_id(), TypeDescriptor::of_in::<String>(context).type_id());

        let nested = TypeDescriptor::of_in::<Option<Box<Option<String>>>>(context);
        let (value, was_optional) = value_descriptor(nested);
        assert!(was_optional);
        assert_eq!(value.type_id(), TypeDescriptor::of_in::<String>(context).type_id());
        assert!(generic_type_argument(value, 0).is_none());
    }
}
