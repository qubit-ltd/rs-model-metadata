// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeKind;

use crate::relation::PropertyPath;

/// Field-local uniqueness semantics.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::FieldUniqueMetadata;
///
/// let unique = FieldUniqueMetadata::for_definition(&[], None);
/// assert_eq!(unique.effective_ignore_case(), None);
/// assert!(!unique.is_scoped());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniqueMetadata {
    /// Property paths that scope uniqueness with this field.
    respect_to: &'static [PropertyPath<'static>],
    /// Whether textual comparisons use case-insensitive matching.
    ignore_case: bool,
    /// Explicit source option; absent when text capability supplies the
    /// default.
    declared_ignore_case: Option<bool>,
    /// Whether the effective default awaits concrete generic field
    /// capabilities.
    deferred: bool,
}

impl UniqueMetadata {
    /// Creates uniqueness metadata.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths that constrain the uniqueness key.
    /// * `ignore_case` - Whether textual comparisons ignore case.
    ///
    /// # Returns
    ///
    /// Concrete uniqueness metadata with the supplied comparison policy.
    #[must_use]
    #[inline]
    pub const fn new(respect_to: &'static [PropertyPath<'static>], ignore_case: bool) -> Self {
        Self {
            respect_to,
            ignore_case,
            declared_ignore_case: Some(ignore_case),
            deferred: false,
        }
    }

    /// Preserves a source-level generic declaration before concrete
    /// substitution.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths declared on the generic definition.
    /// * `ignore_case` - Explicit policy, or `None` for a deferred default.
    ///
    /// # Returns
    ///
    /// Unresolved metadata retaining the declaration for later substitution.
    #[must_use]
    #[inline]
    pub const fn for_definition(respect_to: &'static [PropertyPath<'static>], ignore_case: Option<bool>) -> Self {
        Self {
            respect_to,
            ignore_case: false,
            declared_ignore_case: ignore_case,
            deferred: true,
        }
    }

    /// Creates metadata for a field type that may be resolved later.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths that constrain the uniqueness key.
    /// * `ignore_case` - Explicit comparison policy, when provided.
    ///
    /// # Returns
    ///
    /// Metadata with the explicit policy or a deferred default policy.
    #[must_use]
    pub const fn for_type(respect_to: &'static [PropertyPath<'static>], ignore_case: Option<bool>) -> Self {
        Self {
            respect_to,
            ignore_case: match ignore_case {
                Some(value) => value,
                None => false,
            },
            declared_ignore_case: ignore_case,
            deferred: ignore_case.is_none(),
        }
    }

    /// Binds the inferred comparison policy to the concrete field type in a
    /// context.
    ///
    /// # Parameters
    ///
    /// * `field` - Field metadata whose concrete type supplies the default.
    /// * `context` - Reflection context used to resolve the field type.
    ///
    /// # Returns
    ///
    /// The explicitly declared policy, or whether the concrete field type is
    /// textual when the policy was omitted.
    #[must_use]
    pub fn effective_ignore_case_in(
        &self,
        field: &crate::metadata::FieldMetadata,
        context: &ReflectionContext,
    ) -> bool {
        self.declared_ignore_case.unwrap_or_else(|| {
            field
                .reflect_in(context)
                .and_then(|field| field.find_concrete_type())
                .is_some_and(is_textual_or_transparent)
        })
    }

    /// Returns the explicit declaration independently of the effective default.
    ///
    /// # Returns
    ///
    /// The declared option, preserving whether the policy was omitted.
    #[inline]
    pub const fn declared_ignore_case(&self) -> Option<bool> {
        self.declared_ignore_case
    }

    /// Returns an explicit policy even before generic substitution. Without
    /// an explicit option, returns None for a definition and Some of the
    /// inferred policy for a concrete field.
    ///
    /// # Returns
    ///
    /// The explicit option for deferred definitions, or the effective policy
    /// for concrete metadata.
    #[inline]
    pub const fn effective_ignore_case(&self) -> Option<bool> {
        if self.deferred {
            self.declared_ignore_case
        } else {
            Some(self.ignore_case)
        }
    }

    /// Returns scope paths in source order.
    ///
    /// # Returns
    ///
    /// The declared uniqueness scope paths without reordering.
    #[must_use]
    #[inline]
    pub const fn respect_to(&self) -> &'static [PropertyPath<'static>] {
        self.respect_to
    }

    /// Returns the stored concrete comparison policy. Generic definitions
    /// retain a false placeholder here; use [`Self::effective_ignore_case`] to
    /// distinguish unresolved defaults and preserve explicit source options.
    ///
    /// # Returns
    ///
    /// The stored case-comparison policy, including a placeholder for deferred
    /// generic defaults.
    #[must_use]
    #[inline]
    pub const fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    /// Returns whether uniqueness is scoped by another property.
    ///
    /// # Returns
    ///
    /// `true` when at least one scope path is present.
    #[must_use]
    #[inline]
    pub const fn is_scoped(&self) -> bool {
        !self.respect_to.is_empty()
    }
}

/// Reports whether a reflected type is textual after supported wrappers are
/// expanded.
///
/// # Parameters
///
/// * `descriptor` - Concrete field type to inspect, including any wrappers.
///
/// # Returns
///
/// `true` when the type is text or an optional/smart-pointer chain ending in
/// text.
fn is_textual_or_transparent(mut descriptor: qubit_reflect::TypeDescriptor<'_>) -> bool {
    loop {
        match descriptor.kind() {
            TypeKind::Text(_) => return true,
            TypeKind::Optional(_) | TypeKind::SmartPointer(_) => {
                let Some(argument) = descriptor.generic_arguments().get_at(0) else {
                    return false;
                };
                let Ok(argument) = argument.type_argument() else {
                    return false;
                };
                let Some(inner) = argument.find_concrete_type() else {
                    return false;
                };
                descriptor = inner;
            }
            _ => return false,
        }
    }
}
