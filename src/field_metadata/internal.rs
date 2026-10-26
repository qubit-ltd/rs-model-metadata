// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private helpers for interpreting retained field type expressions.

use qubit_reflect::__private::codegen_v4::expression::ExpressionData;
use qubit_reflect::__private::codegen_v4::expression::GenericExpressionArgument;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;

/// Recursively checks whether an expression contains the special `Self` type.
///
/// # Parameters
///
/// * `expression` - The static type expression to inspect.
///
/// # Returns
///
/// `true` when the expression contains `Self`; otherwise `false`.
#[must_use]
pub(super) fn contains_self_type(expression: &TypeExpressionData) -> bool {
    /// Checks type arguments and associated-type values for `Self`.
    ///
    /// # Parameters
    ///
    /// * `argument` - The generic argument to inspect.
    ///
    /// # Returns
    ///
    /// `true` when the argument contains `Self`; otherwise `false`.
    #[must_use]
    fn argument_contains_self(argument: &GenericExpressionArgument) -> bool {
        match argument {
            GenericExpressionArgument::Type(expression)
            | GenericExpressionArgument::AssociatedType { value: expression, .. } => contains_self_type(expression),
            GenericExpressionArgument::Lifetime(_)
            | GenericExpressionArgument::Const(_)
            | GenericExpressionArgument::AssociatedTypeBound { .. } => false,
        }
    }

    match expression {
        ExpressionData::SelfType => true,
        ExpressionData::Named { arguments, .. } => arguments.iter().any(argument_contains_self),
        ExpressionData::AssociatedType {
            self_type, arguments, ..
        } => contains_self_type(self_type) || arguments.iter().any(argument_contains_self),
        ExpressionData::Reference { target, .. }
        | ExpressionData::RawPointer { target, .. }
        | ExpressionData::Slice(target) => contains_self_type(target),
        ExpressionData::Array { element, .. } => contains_self_type(element),
        ExpressionData::Tuple(elements) => elements.iter().any(|element| contains_self_type(element)),
        ExpressionData::FunctionPointer(function) => {
            function.arguments.iter().any(|argument| contains_self_type(argument))
                || function.result.is_some_and(contains_self_type)
        }
        ExpressionData::Parameter(_)
        | ExpressionData::ParameterName(_)
        | ExpressionData::TraitObject(_)
        | ExpressionData::Opaque(_)
        | ExpressionData::Never => false,
    }
}
