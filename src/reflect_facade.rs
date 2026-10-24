// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-specific access to the shared reflection descriptor root.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

pub(crate) use internal::ModelImplResolution;
use qubit_reflect::ReflectionContext;
use qubit_reflect::capability::CapabilityRegistration;
use qubit_reflect::capability::CapabilitySpec;
use qubit_reflect::descriptor::Reflect;

#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::ModelImplMetadata;
use crate::metadata::TypeMetadata;

// Owns private resolution state shared with reflected model metadata.
#[path = "reflect_facade/internal/model_impl_resolution.rs"]
mod internal;

const MODEL_IMPL_BASE_ID: &str = "qubit.model.impl.v1";

/// The typed capability adapter supplied by generated model declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::ModelMetadataProvider;
///
/// fn provider_slot(provider: ModelMetadataProvider) -> ModelMetadataProvider {
///     provider
/// }
/// ```
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

/// Typed capability contract for generated concrete model metadata.
pub struct ModelMetadataCapability;

impl CapabilitySpec for ModelMetadataCapability {
    type Adapter = ModelMetadataProvider;
    const ID: &'static str = "qubit.model.metadata.v1";
}

/// The typed capability adapter supplied by `ModelImpl`.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelImplMetadata;
/// use qubit_model_metadata::__private::ModelImplProvider;
///
/// fn provider_slot(provider: ModelImplProvider) -> ModelImplProvider {
///     let _: fn() -> &'static ModelImplMetadata = provider;
///     provider
/// }
/// ```
#[doc(hidden)]
pub type ModelImplProvider = fn() -> &'static ModelImplMetadata;

/// Stable source coordinates for one independently declared model impl.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelImplSource {
    /// Rust module containing the declaration.
    pub module_path: &'static str,
    /// Source file containing the declaration.
    pub file: &'static str,
    /// One-based source line of the declaration.
    pub line: u32,
    /// Stable fingerprint distinguishing declarations on the same line.
    pub fingerprint: u64,
}

/// Inventory record for one independently declared model implementation.
#[derive(Clone, Copy)]
pub struct ModelImplFragment {
    target_type_id: fn() -> TypeId,
    provider: ModelImplProvider,
    source: ModelImplSource,
}

impl ModelImplFragment {
    /// Creates a source-attributed fragment for target type `T`.
    #[doc(hidden)]
    pub const fn for_type<T: Reflect + 'static>(provider: ModelImplProvider, source: ModelImplSource) -> Self {
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
    pub const fn provider(self) -> ModelImplProvider {
        self.provider
    }

    /// Returns the source coordinates retained for diagnostics.
    pub const fn source(self) -> ModelImplSource {
        self.source
    }
}

qubit_reflect::__private::__inventory::collect!(ModelImplFragment);

/// Submits one source-attributed `ModelImpl` provider to the model inventory.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_submit_model_impl_fragment {
    ($target:ty, $provider:expr, $fingerprint:expr $(,)?) => {
        $crate::__private::__inventory::submit! {
            $crate::__private::ModelImplFragment::for_type::<$target>(
                $provider,
                $crate::__private::ModelImplSource {
                    module_path: module_path!(),
                    file: file!(),
                    line: line!(),
                    fingerprint: $fingerprint,
                },
            )
        }
    };
}

/// Returns the linked implementation fragments for `T` in inventory order.
#[doc(hidden)]
pub fn model_impl_fragments<T: Reflect + 'static>() -> &'static [ModelImplFragment] {
    static FRAGMENTS: OnceLock<Mutex<HashMap<TypeId, &'static [ModelImplFragment]>>> = OnceLock::new();
    let fragments = FRAGMENTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut fragments = fragments.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    fragments.entry(TypeId::of::<T>()).or_insert_with(|| {
        let selected: Vec<_> = all_model_impl_fragments()
            .iter()
            .copied()
            .filter(|fragment| (fragment.target_type_id)() == TypeId::of::<T>())
            .collect();
        Box::leak(selected.into_boxed_slice())
    })
}

/// Returns all linked model implementation source fragments.
#[must_use]
pub fn all_model_impl_fragments() -> &'static [ModelImplFragment] {
    static FRAGMENTS: OnceLock<&'static [ModelImplFragment]> = OnceLock::new();
    FRAGMENTS.get_or_init(|| {
        let fragments = qubit_reflect::__private::__inventory::iter::<ModelImplFragment>
            .into_iter()
            .copied()
            .collect::<Vec<_>>();
        Box::leak(fragments.into_boxed_slice())
    })
}

/// Typed capability contract for generated model implementation overlays.
pub struct ModelImplCapability;

impl CapabilitySpec for ModelImplCapability {
    type Adapter = &'static [ModelImplFragment];
    const ID: &'static str = MODEL_IMPL_BASE_ID;
}

/// The typed capability adapter supplied by generic model declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::generic::GenericModelMetadata;
///
/// fn provider_slot(provider: fn() -> &'static GenericModelMetadata) -> fn() -> &'static GenericModelMetadata {
///     provider
/// }
/// ```
#[doc(hidden)]
#[cfg(feature = "generic")]
pub type GenericModelMetadataProvider = fn() -> &'static GenericModelMetadata;

/// Typed capability contract for generic model definition metadata.
#[doc(hidden)]
#[cfg(feature = "generic")]
pub struct GenericModelMetadataCapability;

#[cfg(feature = "generic")]
impl CapabilitySpec for GenericModelMetadataCapability {
    type Adapter = GenericModelMetadataProvider;
    const ID: &'static str = "qubit.model.generic_metadata.v1";
}

/// Creates the typed registration for a generated concrete model.
#[doc(hidden)]
#[must_use]
pub fn model_metadata_registration<T: crate::metadata::HasTypeMetadata>() -> CapabilityRegistration {
    fn provide<T: crate::metadata::HasTypeMetadata>() -> &'static TypeMetadata {
        let metadata = <T as crate::__private::TypeMetadataProvider>::__type_metadata();
        metadata.assert_valid_for::<T>();
        metadata
    }
    CapabilityRegistration::for_type::<T, ModelMetadataCapability>(provide::<T> as ModelMetadataProvider)
}

/// Creates the typed registration for generated `ModelImpl` metadata.
#[doc(hidden)]
#[must_use]
pub fn model_impl_registration<T: Reflect + 'static>() -> CapabilityRegistration {
    model_impl_capability_registration::<T>(model_impl_fragments::<T>())
}

/// Creates the fixed model implementation capability for a supplied fragment
/// aggregate.
#[doc(hidden)]
#[must_use]
pub fn model_impl_capability_registration<T: Reflect + 'static>(
    fragments: &'static [ModelImplFragment],
) -> CapabilityRegistration {
    CapabilityRegistration::for_type::<T, ModelImplCapability>(fragments)
}

/// Creates the typed registration for generic model metadata.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_metadata_registration(
    definition: qubit_reflect::descriptor::TypeDefinitionId,
    provider: GenericModelMetadataProvider,
) -> CapabilityRegistration {
    CapabilityRegistration::for_definition::<GenericModelMetadataCapability>(definition, provider)
}

/// Builds a capability for one generic model declaration.
///
/// # Parameters
///
/// - `provider`: function returning the declaration's static generic metadata.
///
/// # Returns
///
/// A capability descriptor associating the generic metadata key with
/// `provider`.
#[doc(hidden)]
#[must_use]
#[cfg(feature = "generic")]
pub fn generic_model_capability(
    definition: qubit_reflect::descriptor::TypeDefinitionId,
    provider: GenericModelMetadataProvider,
) -> CapabilityRegistration {
    generic_model_metadata_registration(definition, provider)
}

/// Builds an inline model capability for one concrete reflection monomorph.
///
/// # Type Parameters
///
/// - `T`: concrete reflected type implementing `HasTypeMetadata`.
///
/// # Returns
///
/// A capability descriptor whose adapter validates and returns `T`'s metadata.
#[doc(hidden)]
#[must_use]
pub fn model_capability<T: crate::metadata::HasTypeMetadata>() -> CapabilityRegistration {
    model_metadata_registration::<T>()
}

/// Returns the generated model-implementation overlay attached to an exact
/// descriptor root in the frozen reflection snapshot.
///
/// A concrete type capability takes precedence over its generic definition
/// capability, including when its fragment aggregate has no entry for this
/// exact type. Generic definition providers are used only when the concrete
/// descriptor has no `ModelImplCapability` registration.
///
/// # Parameters
///
/// - `metadata`: validated metadata for the descriptor being resolved.
/// - `registry`: frozen registry containing its model implementation
///   capabilities.
///
/// # Returns
///
/// `None` when no provider is registered, otherwise static metadata for one
/// provider or an owned merge for multiple providers.
///
/// # Errors
///
/// Returns a capability access error when the registry cannot resolve the
/// descriptor's capabilities or their origins.
///
/// # Panics
///
/// Panics if a resolved model implementation capability has no retained
/// registration origin.
// Keep the reflected runtime's complete typed capability error intact at
// this internal boundary; callers wrap it in their own domain error.
#[allow(clippy::result_large_err)]
pub(crate) fn model_impl_metadata(
    metadata: &TypeMetadata,
    context: &ReflectionContext,
) -> Result<Option<ModelImplResolution>, qubit_reflect::capability::CapabilityError> {
    let descriptor = metadata.descriptor_in(context);
    let mut providers = Vec::new();
    let concrete_capability = descriptor.find_capability::<ModelImplCapability>()?;
    if let Some(capability) = &concrete_capability {
        for fragment in capability
            .adapter()
            .iter()
            .filter(|fragment| (fragment.target_type_id)() == metadata.type_id())
        {
            providers.push((fragment.provider(), fragment.source()));
        }
    }
    if concrete_capability.is_none()
        && let Some(definition) = descriptor.find_definition()
        && let Some(capability) = definition.find_capability::<ModelImplCapability>()?
    {
        for fragment in capability
            .adapter()
            .iter()
            .filter(|fragment| (fragment.target_type_id)() == metadata.type_id())
        {
            providers.push((fragment.provider(), fragment.source()));
        }
    }
    if providers.is_empty() {
        return Ok(None);
    }
    Ok(Some(ModelImplResolution::Merged(Arc::new(ModelImplMetadata::merge(
        metadata, &providers, context,
    )))))
}

#[cfg(test)]
mod tests {
    use super::ModelImplFragment;
    use super::ModelImplMetadata;
    use super::ModelImplProvider;
    use super::ModelImplSource;
    use super::model_impl_fragments;

    struct AggregatedFixture;

    fn first_provider() -> &'static ModelImplMetadata {
        panic!("aggregation test does not invoke providers")
    }

    fn second_provider() -> &'static ModelImplMetadata {
        panic!("aggregation test does not invoke providers")
    }

    qubit_reflect::__private::__inventory::submit! {
        ModelImplFragment::for_type::<AggregatedFixture>(
            first_provider as ModelImplProvider,
            ModelImplSource {
                module_path: "reflect_facade::tests",
                file: file!(),
                line: line!(),
                fingerprint: 1,
            },
        )
    }

    qubit_reflect::__private::__inventory::submit! {
        ModelImplFragment::for_type::<AggregatedFixture>(
            second_provider as ModelImplProvider,
            ModelImplSource {
                module_path: "reflect_facade::tests",
                file: file!(),
                line: line!(),
                fingerprint: 2,
            },
        )
    }

    #[test]
    fn collects_multiple_impl_providers_for_one_concrete_type() {
        let fragments = model_impl_fragments::<AggregatedFixture>();
        assert_eq!(fragments.len(), 2);
        let mut fingerprints: Vec<_> = fragments.iter().map(|fragment| fragment.source().fingerprint).collect();
        fingerprints.sort_unstable();
        assert_eq!(fingerprints, [1, 2]);
    }
}
