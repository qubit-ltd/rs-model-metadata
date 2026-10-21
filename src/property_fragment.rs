// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One unmerged field, getter, or setter property declaration.

use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeExpression;

use crate::metadata::PropertyFragmentSource;

/// Preserves one local source fact before property compatibility is checked.
///
/// The type exposes fragments through generated model metadata. Use the
/// accessors to inspect the property name, declared type, and source
/// declaration.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertyFragment;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model(id = "example.PropertyFragment")]
/// struct Note {
///     title: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = TypeMetadata::of::<Note>();
/// let fragments = metadata.property_fragments()?;
/// let title = fragments
///     .fragments()
///     .iter()
///     .find(|fragment: &&PropertyFragment| fragment.name() == "title")
///     .expect("generated title fragment");
/// assert_eq!(title.name(), "title");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct PropertyFragment {
    /// Canonical public property name.
    name: &'static str,
    /// Exact type declared by this fragment.
    type_expression: Option<&'static TypeExpressionData>,
    /// Field/getter/setter declaration source.
    source: PropertyFragmentSource,
}

impl fmt::Debug for PropertyFragment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PropertyFragment")
            .field("name", &self.name)
            .field("has_type_expression", &self.type_expression.is_some())
            .field("source", &self.source)
            .finish()
    }
}

impl PropertyFragment {
    /// Creates one generated property source fact.
    ///
    /// # Parameters
    ///
    /// - `name`: canonical public property name.
    /// - `type_ref`: exact reflected type of the property.
    /// - `source`: declaration category and metadata.
    ///
    /// # Returns
    ///
    /// A fragment retaining the supplied static type and declaration metadata.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        name: &'static str,
        type_expression: Option<&'static TypeExpressionData>,
        source: PropertyFragmentSource,
    ) -> Self {
        Self {
            name,
            type_expression,
            source,
        }
    }

    /// Returns the canonical public property name.
    ///
    /// # Returns
    ///
    /// The name shared by the field and its accessors.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Binds this source fragment's type expression to `context`.
    #[must_use]
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        match self.source {
            PropertyFragmentSource::Field(field) => field.type_expression_in(context),
            PropertyFragmentSource::Getter(getter) => Some(getter.output_type_in(context)),
            PropertyFragmentSource::Setter(setter) => Some(setter.input_type_in(context)),
        }
        .or_else(|| {
            self.type_expression
                .map(|data| qubit_reflect::__private::codegen_v4::expression::type_expression_in(data, context))
        })
    }

    /// Returns the field, getter, or setter that declared this fragment.
    ///
    /// # Returns
    ///
    /// The source category and its corresponding declaration metadata.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> PropertyFragmentSource {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::descriptor::TypeDescriptor;

    use super::PropertyFragment;
    use crate::field_metadata::FieldMetadata;
    use crate::metadata::PropertyFragmentSource;

    #[Reflect]
    struct FragmentCoverageFixture {
        name: String,
    }

    #[test]
    fn debug_and_unbound_type_expression_fallback_are_observable() {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<FragmentCoverageFixture>(context)
            .field("name")
            .expect("fixture field");
        let field = Box::leak(Box::new(FieldMetadata::from_reflect(
            std::any::TypeId::of::<FragmentCoverageFixture>(),
            descriptor,
        )));
        let fragment = PropertyFragment::new("name", None, PropertyFragmentSource::Field(field));
        assert!(format!("{fragment:?}").contains("PropertyFragment"));
        let _ = fragment.type_expression_in(context);
    }
}
