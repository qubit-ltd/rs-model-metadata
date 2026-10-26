// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::relation::DeclarationLocation;
use crate::relation::ObjectPath;
use crate::relation::PropertyPath;

/// Names one validator dependency slot and the property path supplying it.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::DependencyBindingMetadata;
/// use qubit_model_metadata::metadata::PropertyPath;
///
/// let binding = DependencyBindingMetadata::new("owner", PropertyPath::new(&["owner"]));
/// assert_eq!(binding.name(), "owner");
/// assert_eq!(binding.path().to_string(), "owner");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencyBindingMetadata {
    /// The name used by the validator signature.
    name: &'static str,
    /// The model property path supplying this dependency.
    path: PropertyPath<'static>,
    /// Navigation from the owning object before selecting the property.
    object_path: ObjectPath,
    /// Source occurrence anchoring navigation to its owning object.
    declaration: &'static DeclarationLocation,
}

impl DependencyBindingMetadata {
    /// Creates a named dependency binding.
    ///
    /// # Parameters
    ///
    /// * `name` - Non-empty validator signature slot name.
    /// * `path` - Non-empty model property path supplying the slot.
    ///
    /// # Returns
    ///
    /// A dependency binding with local object navigation by default.
    ///
    /// # Panics
    /// Panics if the slot name or property path is empty.
    #[must_use]
    #[inline]
    pub const fn new(name: &'static str, path: PropertyPath<'static>) -> Self {
        static UNKNOWN: DeclarationLocation = DeclarationLocation::unknown();
        assert!(!name.is_empty(), "validator dependency name cannot be empty");
        assert!(!path.is_empty(), "validator dependency path cannot be empty");
        Self {
            name,
            path,
            object_path: ObjectPath::current(),
            declaration: &UNKNOWN,
        }
    }

    /// Returns navigation relative to the declaration's owning object.
    ///
    /// # Returns
    ///
    /// The object path used before selecting the dependency property.
    #[must_use]
    #[inline]
    pub const fn object_path(&self) -> ObjectPath {
        self.object_path
    }

    /// Returns the property selected on the navigated object.
    ///
    /// # Returns
    ///
    /// The model property path supplying this dependency slot.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> PropertyPath<'static> {
        self.path
    }

    /// Returns source and owning-object coordinates for this dependency.
    ///
    /// # Returns
    ///
    /// The declaration location associated with this binding.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static DeclarationLocation {
        self.declaration
    }

    /// Returns the validator signature slot name.
    ///
    /// # Returns
    ///
    /// The stable parameter name borrowed from static metadata.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the model property path supplying the slot.
    ///
    /// # Returns
    ///
    /// The static path selected for this dependency slot.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> PropertyPath<'static> {
        self.path
    }

    /// Adds object navigation without changing the selected property.
    ///
    /// # Parameters
    ///
    /// * `path` - Navigation from the declaration owner to the property owner.
    ///
    /// # Returns
    ///
    /// This binding with the supplied navigation path.
    #[must_use]
    #[inline]
    pub const fn with_object_path(mut self, path: ObjectPath) -> Self {
        self.object_path = path;
        self
    }

    /// Associates an exact field or selector occurrence with this binding.
    ///
    /// # Parameters
    ///
    /// * `declaration` - Source location identifying the owning occurrence.
    ///
    /// # Returns
    ///
    /// This binding associated with the supplied declaration location.
    #[must_use]
    #[inline]
    pub const fn with_declaration(mut self, declaration: &'static DeclarationLocation) -> Self {
        self.declaration = declaration;
        self
    }
}
