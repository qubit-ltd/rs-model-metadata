// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::collections::HashSet;

use qubit_reflect::__private::codegen_v4::expression::type_expression_in;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::TypeKind;

use super::contains_field;
use crate::metadata::FieldMetadata;
use crate::metadata::GetterMetadata;
use crate::metadata::GetterOutputKind;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyMetadata;

/// Resolves a type expression's concrete type ID when available.
///
/// # Parameters
///
/// - `property`: Property whose reflected field or declared type is resolved.
/// - `fields`: Owning type's fields used to find a backing field by property
///   name.
/// - `context`: Reflection context used to resolve the type expression.
fn property_type_descriptor<'ctx>(
    property: &PropertyMetadata,
    fields: &[FieldMetadata],
    context: &'ctx ReflectionContext,
) -> Option<TypeDescriptor<'ctx>> {
    property
        .field()
        .or_else(|| fields.iter().find(|field| field.name() == Some(property.name())))?
        .reflect_in(context)?
        .find_concrete_type()
}

/// Checks whether an explicit property expression agrees with an accessor
/// signature when no concrete backing field can establish the type identity.
///
/// # Parameters
///
/// - `property`: Property whose declared type expression is compared.
/// - `accessor`: Type expression from the accessor signature.
/// - `context`: Reflection context used to resolve the property's expression.
#[must_use]
fn property_accessor_expression_matches(
    property: &PropertyMetadata,
    accessor: TypeExpression<'_>,
    context: &ReflectionContext,
) -> bool {
    let Some(property_expression) = property.type_expression_in(context) else {
        return true;
    };
    property_expression.structurally_eq(accessor)
}

/// Checks whether a getter's concrete output agrees with the property type.
///
/// # Parameters
///
/// - `property_type`: Reflected property type, when it can be resolved.
/// - `getter`: Accessor metadata supplying the output type and shape.
/// - `context`: Reflection context used to resolve output and element types.
#[must_use]
fn getter_type_compatible(
    property_type: Option<TypeDescriptor<'_>>,
    getter: &GetterMetadata,
    context: &ReflectionContext,
) -> bool {
    let Some(property_type) = property_type else {
        return true;
    };
    let output_type_id = getter.output_type_id();
    if property_type.type_id() == output_type_id {
        return true;
    }
    if text_accessor_type_compatible(property_type, output_type_id, context) {
        return true;
    }
    if getter.output_kind() == GetterOutputKind::BorrowedSlice {
        let matches = property_type
            .generic_arguments()
            .get_at(0)
            .and_then(|argument| argument.type_argument().ok())
            .and_then(|argument| argument.find_concrete_type())
            .zip(getter.output_element_type_id())
            .is_some_and(|(element, output)| element.type_id() == output);
        return matches;
    }
    let Some(output_type) = context.find_type_by_id(output_type_id).ok().flatten() else {
        return false;
    };
    if matches!(
        (property_type.kind(), output_type.kind()),
        (TypeKind::Text(_), TypeKind::Text(_))
    ) {
        return true;
    }
    let property_element = match property_type.kind() {
        TypeKind::Sequence(_) | TypeKind::Array(_) => property_type.generic_arguments().get_at(0),
        _ => None,
    }
    .and_then(|argument| argument.type_argument().ok())
    .and_then(|argument| argument.find_concrete_type());
    let output_element = match output_type.kind() {
        TypeKind::Slice(_) => output_type.generic_arguments().get_at(0),
        _ => None,
    }
    .and_then(|argument| argument.type_argument().ok())
    .and_then(|argument| argument.find_concrete_type());
    property_element
        .zip(output_element)
        .is_some_and(|(a, b)| a.type_id() == b.type_id())
}

/// Checks whether an accessor type is a text-compatible view of a property.
///
/// # Parameters
///
/// - `property_type`: Reflected property type whose text kind is checked.
/// - `accessor_type_id`: Concrete type identity reported by the accessor.
/// - `context`: Reflection context used to inspect the accessor type.
#[must_use]
fn text_accessor_type_compatible(
    property_type: TypeDescriptor<'_>,
    accessor_type_id: TypeId,
    context: &ReflectionContext,
) -> bool {
    if !matches!(property_type.kind(), TypeKind::Text(_)) {
        return false;
    }
    if accessor_type_id == property_type.type_id() {
        return true;
    }
    if property_type.type_id() == TypeId::of::<String>() && accessor_type_id == TypeId::of::<str>() {
        return true;
    }
    context
        .find_type_by_id(accessor_type_id)
        .ok()
        .flatten()
        .is_some_and(|accessor_type| matches!(accessor_type.kind(), TypeKind::Text(_)))
}

/// Checks whether two accessor identities agree for one property type.
///
/// # Parameters
///
/// - `property_type`: Reflected property type used to allow compatible text
///   views.
/// - `left_type_id`: Concrete output type identity of the first accessor.
/// - `right_type_id`: Concrete output type identity of the second accessor.
/// - `context`: Reflection context used to resolve text-compatible accessor
///   types.
#[must_use]
fn accessor_types_compatible(
    property_type: Option<TypeDescriptor<'_>>,
    left_type_id: TypeId,
    right_type_id: TypeId,
    context: &ReflectionContext,
) -> bool {
    left_type_id == right_type_id
        || property_type.is_some_and(|property_type| {
            text_accessor_type_compatible(property_type, left_type_id, context)
                && text_accessor_type_compatible(property_type, right_type_id, context)
        })
}

/// Verifies that generated property overlays agree with fields and reflection.
///
/// # Parameters
///
/// - `properties`: Property overlays to validate for names, sources, and type
///   agreement.
/// - `fields`: Reflected fields owned by the type described by `descriptor`.
/// - `descriptor`: Reflected owner type and context used to validate the
///   overlays.
pub(in crate::type_metadata) fn validate_properties(
    properties: &[PropertyMetadata],
    fields: &[FieldMetadata],
    descriptor: TypeDescriptor<'_>,
) -> Result<(), PropertyBuildErrors> {
    let mut names = HashSet::with_capacity(properties.len());
    let mut errors = Vec::new();
    for property in properties {
        if property.name().is_empty() || !names.insert(property.name()) {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::InvalidName,
                property.name(),
            ));
        }
        if property.field().is_none() && property.getter().is_none() && property.setter().is_none() {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::MissingSource,
                property.name(),
            ));
        }
        if let Some(field) = property.field() {
            let reflected = field.reflect_in(descriptor.context());
            let is_owned_field = contains_field(fields, field);
            if !is_owned_field
                || (!field.is_opaque()
                    && reflected
                        .as_ref()
                        .is_some_and(|reflected| reflected.declaring_type().type_id() != descriptor.type_id()))
            {
                errors.push(PropertyBuildError::new(
                    PropertyBuildErrorKind::ForeignField,
                    property.name(),
                ));
            }
            if is_owned_field && !field.is_opaque() && property.getter().is_none() && property.setter().is_none() {
                let expression_mismatch = property
                    .declared_type_expression_in(descriptor.context())
                    .zip(
                        field
                            .type_expression_data()
                            .map(|expression| type_expression_in(expression, descriptor.context())),
                    )
                    .is_some_and(|(declared, field)| !declared.structurally_eq(field));
                if expression_mismatch {
                    errors.push(PropertyBuildError::new(
                        PropertyBuildErrorKind::FieldTypeMismatch,
                        property.name(),
                    ));
                }
            }
        }
        if let Some(getter) = property.getter()
            && (getter.target_type_id() != descriptor.type_id()
                || (property.field().is_none()
                    && !property_accessor_expression_matches(
                        property,
                        getter.output_type_in(descriptor.context()),
                        descriptor.context(),
                    ))
                || !getter_type_compatible(
                    property_type_descriptor(property, fields, descriptor.context()),
                    getter,
                    descriptor.context(),
                ))
        {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::GetterTypeMismatch,
                property.name(),
            ));
        }
        if let Some(setter) = property.setter()
            && (setter.target_type_id() != descriptor.type_id()
                || (property.field().is_none()
                    && property.getter().is_none()
                    && !property_accessor_expression_matches(
                        property,
                        setter.input_type_in(descriptor.context()),
                        descriptor.context(),
                    ))
                || property_type_descriptor(property, fields, descriptor.context()).is_some_and(|value| {
                    !accessor_types_compatible(
                        Some(value),
                        value.type_id(),
                        setter.input_type_id(),
                        descriptor.context(),
                    )
                })
                || property.getter().is_some_and(|getter| {
                    !accessor_types_compatible(
                        property_type_descriptor(property, fields, descriptor.context()),
                        getter.output_type_id(),
                        setter.input_type_id(),
                        descriptor.context(),
                    )
                }))
        {
            errors.push(PropertyBuildError::new(
                PropertyBuildErrorKind::SetterTypeMismatch,
                property.name(),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(PropertyBuildErrors::new(errors))
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::value::DynamicRef;
    use qubit_reflect::value::Local;

    use super::accessor_types_compatible;
    use super::getter_type_compatible;
    use super::property_accessor_expression_matches;
    use super::property_type_descriptor;
    use super::text_accessor_type_compatible;
    use crate::__private::v8;
    use crate::metadata::GetterMetadata;
    use crate::metadata::GetterOutputKind;
    use crate::metadata::PropertyAccessError;
    use crate::metadata::PropertyValue;

    static STRING_EXPRESSION: TypeExpressionData = TypeExpressionData::Named {
        path: "alloc::string::String",
        definition: None,
        arguments: &[],
    };

    fn never_get(_: DynamicRef<'_, Local>) -> Result<PropertyValue<'_>, PropertyAccessError> {
        panic!("property compatibility checks must not invoke accessors");
    }

    #[test]
    fn test_text_property_accessor_type_compatibility() {
        let context = ReflectionContext::builder()
            .include_type::<String>()
            .include_type::<str>()
            .build()
            .expect("text property reflection context");
        let getter = v8::leak(GetterMetadata::new::<String>(
            "as_text",
            &STRING_EXPRESSION,
            TypeId::of::<str>,
            GetterOutputKind::Borrowed,
            never_get,
        ));
        let property = v8::property_metadata("text", None, None, Some(getter), None);
        let descriptor = context.type_by_id(TypeId::of::<String>()).expect("String descriptor");

        assert_eq!(descriptor.type_id(), TypeId::of::<String>());
        assert!(property_type_descriptor(&property, &[], &context).is_none());
        assert!(property_accessor_expression_matches(
            &property,
            getter.output_type_in(&context),
            &context,
        ));
        assert!(text_accessor_type_compatible(descriptor, TypeId::of::<str>(), &context));
        assert!(getter_type_compatible(Some(descriptor), getter, &context));
        assert!(accessor_types_compatible(
            Some(descriptor),
            TypeId::of::<String>(),
            TypeId::of::<str>(),
            &context,
        ));
    }
}
