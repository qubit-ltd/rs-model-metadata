// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Source-attributed model implementation providers retained in inventory.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use qubit_reflect::__private::__inventory;
use qubit_reflect::descriptor::Reflect;

use super::ModelImplSource;

/// The typed capability adapter supplied by the generated `ModelImpl`
/// registration flow.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelImplMetadata;
/// use qubit_model_metadata::__private::ModelImplProvider;
///
/// fn inspect_provider(provider: ModelImplProvider) -> &'static ModelImplMetadata {
///     provider()
/// }
/// ```
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static crate::metadata::ModelImplMetadata;

/// Inventory record for one independently declared model implementation.
///
/// Fragments are submitted by the generated model implementation registration
/// flow and collected by the framework; callers should inspect them rather
/// than construct them directly. The public inventory accessor supplies the
/// fragments shown in the example.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::ModelImplFragment;
/// use qubit_model_metadata::__private::v8::all_model_impl_fragments;
///
/// fn inspect_fragments() {
///     for fragment in all_model_impl_fragments() {
///         let provider = fragment.provider();
///         let source = fragment.source();
///         let _registration = (provider, source);
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ModelImplFragment {
    /// Type identity used to select fragments for a concrete reflection type.
    pub(super) target_type_id: fn() -> TypeId,
    /// Provider returning the static metadata for this implementation.
    provider: ModelImplProvider,
    /// Declaration coordinates retained for diagnostics.
    source: ModelImplSource,
}

impl ModelImplFragment {
    /// Creates a source-attributed fragment for target type `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: reflected type whose runtime identity is retained by the
    ///   fragment.
    ///
    /// # Parameters
    ///
    /// - `provider`: function that supplies the implementation metadata.
    /// - `source`: declaration coordinates used for diagnostics.
    ///
    /// # Returns
    ///
    /// A fragment associated with the runtime identity of `T`.
    #[must_use]
    #[inline]
    #[doc(hidden)]
    pub const fn for_type<T: Reflect + 'static>(provider: ModelImplProvider, source: ModelImplSource) -> Self {
        /// Returns the runtime type identity used to filter inventory entries.
        ///
        /// # Returns
        ///
        /// The runtime identity of `T`.
        fn type_id<T: 'static>() -> TypeId {
            TypeId::of::<T>()
        }
        Self {
            target_type_id: type_id::<T>,
            provider,
            source,
        }
    }

    /// Returns the provider function for this fragment.
    ///
    /// # Returns
    ///
    /// The function that supplies this implementation's static metadata.
    #[must_use]
    #[inline]
    pub const fn provider(self) -> ModelImplProvider {
        self.provider
    }

    /// Returns the source coordinates retained for diagnostics.
    ///
    /// # Returns
    ///
    /// The declaration location and stable fingerprint associated with this
    /// fragment.
    #[must_use]
    #[inline]
    pub const fn source(self) -> ModelImplSource {
        self.source
    }
}

__inventory::collect!(ModelImplFragment);

/// Returns the linked implementation fragments for `T` in inventory order.
///
/// The filtered slice is cached by type identity and shared for the process
/// lifetime, so each type's inventory scan and allocation happen once.
///
/// # Type Parameters
///
/// - `T`: reflected type whose implementation fragments are selected.
///
/// # Returns
///
/// The cached fragments whose target type is `T`, in inventory order.
#[doc(hidden)]
#[must_use]
pub fn model_impl_fragments<T: Reflect + 'static>() -> &'static [ModelImplFragment] {
    static FRAGMENTS: OnceLock<Mutex<HashMap<TypeId, &'static [ModelImplFragment]>>> = OnceLock::new();
    let fragments = FRAGMENTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut fragments = fragments.lock().unwrap_or_else(PoisonError::into_inner);
    fragments
        .entry(TypeId::of::<T>())
        .or_insert_with(|| select_model_impl_fragments(TypeId::of::<T>()))
}

/// Selects and retains all inventory fragments matching one runtime type.
///
/// The copied entries preserve inventory order; leaking the owned slice gives
/// the process-wide cache a stable value for the lifetime of the program.
///
/// # Parameters
///
/// - `type_id`: runtime identity used to select matching fragments.
///
/// # Returns
///
/// The selected fragments in inventory order, retained for the process
/// lifetime.
fn select_model_impl_fragments(type_id: TypeId) -> &'static [ModelImplFragment] {
    let selected: Vec<_> = all_model_impl_fragments()
        .iter()
        .copied()
        .filter(|fragment| (fragment.target_type_id)() == type_id)
        .collect();
    Box::leak(selected.into_boxed_slice())
}

/// Returns all linked model implementation source fragments in inventory
/// order.
///
/// The slice is initialized once and shared for the process lifetime.
///
/// # Returns
///
/// All linked fragments in inventory order.
#[must_use]
pub fn all_model_impl_fragments() -> &'static [ModelImplFragment] {
    static FRAGMENTS: OnceLock<&'static [ModelImplFragment]> = OnceLock::new();
    FRAGMENTS.get_or_init(|| {
        let fragments = collect_inventory_fragments();
        Box::leak(fragments.into_boxed_slice())
    })
}

/// Copies all inventory records into an owned vector in registration order.
fn collect_inventory_fragments() -> Vec<ModelImplFragment> {
    __inventory::iter::<ModelImplFragment>.into_iter().copied().collect()
}
