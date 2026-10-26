// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed capability for generated model implementation fragments.

use qubit_reflect::capability::CapabilitySpec;

use super::ModelImplFragment;

/// Stable identifier used to locate model implementation capabilities.
///
/// This identifier is shared with generated model implementation registrations.
const MODEL_IMPL_BASE_ID: &str = "qubit.model.impl.v1";

/// Typed capability contract for generated model implementation overlays.
///
/// The model derive flow registers this capability with the linked
/// implementation fragments for each generated model.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::v8::ModelImplCapability;
/// use qubit_reflect::capability::CapabilitySpec;
///
/// let capability_id = <ModelImplCapability as CapabilitySpec>::ID;
/// let fragments: <ModelImplCapability as CapabilitySpec>::Adapter = &[];
/// assert_eq!(capability_id, "qubit.model.impl.v1");
/// assert!(fragments.is_empty());
/// ```
pub struct ModelImplCapability;

impl CapabilitySpec for ModelImplCapability {
    type Adapter = &'static [ModelImplFragment];
    const ID: &'static str = MODEL_IMPL_BASE_ID;
}
