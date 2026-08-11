// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Copyable references to static type-shape metadata.

use core::any::type_name;

use crate::type_metadata::TypeIdentity;
use crate::type_metadata::TypeMetadata;
use crate::type_shape::HasTypeShape;
use crate::type_shape::TypeCapabilities;
use crate::type_shape::TypeShape;

/// A small, copyable reference to a type's static shape metadata.
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct TypeRef {
    /// The runtime identity of the referenced Rust type.
    identity: TypeIdentity,
    /// A function that returns the fully qualified name of the referenced
    /// type.
    type_name: fn() -> &'static str,
    /// A function that returns the referenced type's structural shape.
    shape: fn() -> TypeShape,
    /// The capabilities of the referenced type's outermost structural layer.
    capabilities: TypeCapabilities,
    /// Capabilities exposed by sequence elements, when applicable.
    element_capabilities: Option<TypeCapabilities>,
}

impl TypeRef {
    /// Creates a reference to the static metadata for `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements [`HasTypeShape`].
    ///
    /// # Returns
    ///
    /// A reference to `T`'s static type shape metadata.
    #[inline]
    pub const fn of<T: HasTypeShape>() -> Self {
        Self {
            identity: TypeIdentity::of::<T>(),
            type_name: type_name::<T>,
            shape: type_shape_of::<T>,
            capabilities: T::CAPABILITIES,
            element_capabilities: T::ELEMENT_CAPABILITIES,
        }
    }

    /// Creates an opaque reference for `T` without requiring [`HasTypeShape`].
    ///
    /// The resulting reference retains `T`'s Rust type name while exposing only
    /// [`TypeShape::Opaque`] to structural queries.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The `'static` Rust type represented by the opaque reference.
    ///
    /// # Returns
    ///
    /// An opaque reference that retains `T`'s type name.
    #[inline]
    pub const fn opaque<T: 'static>() -> Self {
        Self {
            identity: TypeIdentity::of::<T>(),
            type_name: type_name::<T>,
            shape: opaque_type_shape,
            capabilities: TypeCapabilities::NONE,
            element_capabilities: None,
        }
    }

    /// Creates an opaque reference for `T` with a producer-supplied outer
    /// structural shape.
    ///
    /// This is intended for metadata producers that can observe standard
    /// container syntax while intentionally leaving their leaf type opaque.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The complete `'static` Rust type represented by the reference.
    ///
    /// # Parameters
    ///
    /// * `shape` - A function returning the visible structural shape whose
    ///   opaque leaves are represented by [`TypeRef::opaque`].
    ///
    /// # Returns
    ///
    /// An opaque reference that retains `T`'s identity and visible container
    /// structure.
    #[inline]
    pub const fn opaque_with_shape<T: 'static>(
        shape: fn() -> TypeShape,
    ) -> Self {
        Self {
            identity: TypeIdentity::of::<T>(),
            type_name: type_name::<T>,
            shape,
            capabilities: TypeCapabilities::NONE,
            element_capabilities: None,
        }
    }

    /// Returns the recursive shape represented by this reference.
    ///
    /// # Returns
    ///
    /// The structural shape represented by this reference.
    #[inline(always)]
    pub fn shape(self) -> TypeShape {
        (self.shape)()
    }

    /// Returns Rust's fully qualified name for the referenced type.
    ///
    /// # Returns
    ///
    /// The fully qualified Rust type name represented by this reference.
    #[must_use]
    #[inline(always)]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }

    /// Returns the runtime identity of the referenced Rust type.
    ///
    /// # Returns
    ///
    /// The identity used to compare compatible structural leaf types.
    #[must_use]
    #[inline(always)]
    pub const fn identity(self) -> TypeIdentity {
        self.identity
    }

    /// Returns the capabilities supported by the referenced type's outermost
    /// structural layer.
    ///
    /// # Returns
    ///
    /// The capabilities supported by the outermost structural layer.
    #[inline(always)]
    pub const fn capabilities(self) -> TypeCapabilities {
        self.capabilities
    }

    /// Returns the capabilities of sequence elements.
    ///
    /// # Returns
    ///
    /// `Some` with the element capabilities for sequence and array shapes;
    /// otherwise, `None`.
    #[inline(always)]
    pub const fn element_capabilities(self) -> Option<TypeCapabilities> {
        self.element_capabilities
    }

    /// Removes one outer `Option` layer, leaving other shapes unchanged.
    ///
    /// # Returns
    ///
    /// The inner reference when the outer shape is optional; otherwise, this
    /// reference unchanged.
    #[inline]
    pub fn strip_optional(self) -> Self {
        match self.shape() {
            TypeShape::Optional(inner) => inner,
            _ => self,
        }
    }

    /// Resolves an outer named type after removing one optional layer, if
    /// present.
    ///
    /// # Returns
    ///
    /// `Some` with the named type's metadata when the resulting shape is named
    /// and has a resolver; otherwise, `None`.
    #[must_use]
    #[inline]
    pub fn named_metadata(self) -> Option<&'static TypeMetadata> {
        match self.strip_optional().shape() {
            TypeShape::Named(named) => named.metadata(),
            _ => None,
        }
    }
}

/// Returns the shape associated with `T` for use in [`TypeRef`].
///
/// # Type Parameters
///
/// * `T` - The type that implements [`HasTypeShape`].
///
/// # Returns
///
/// The static shape declared by `T`.
#[inline]
fn type_shape_of<T: HasTypeShape>() -> TypeShape {
    T::TYPE_SHAPE
}

/// Returns the intentionally uninterpreted shape associated with opaque `T`.
///
/// # Returns
///
/// [`TypeShape::Opaque`].
#[inline]
fn opaque_type_shape() -> TypeShape {
    TypeShape::Opaque
}
