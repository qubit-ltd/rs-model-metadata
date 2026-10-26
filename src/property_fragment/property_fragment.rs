// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::__private::codegen_v4::expression::type_expression_in;
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
    /// Formats the fragment's name, type-expression presence, and source
    /// metadata.
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
    /// - `type_expression`: optional static type syntax for the property.
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
    ///
    /// Field, getter, and setter metadata take precedence; the retained static
    /// expression is used only when the source declaration has no bound type.
    ///
    /// # Parameters
    ///
    /// - `context`: reflection context used to resolve context-bound type data.
    ///
    /// # Returns
    ///
    /// `Some` when the declaration or retained expression provides a type;
    /// otherwise, `None`.
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        match self.source {
            PropertyFragmentSource::Field(field) => field.type_expression_in(context),
            PropertyFragmentSource::Getter(getter) => Some(getter.output_type_in(context)),
            PropertyFragmentSource::Setter(setter) => Some(setter.input_type_in(context)),
        }
        .or_else(|| self.type_expression.map(|data| type_expression_in(data, context)))
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
