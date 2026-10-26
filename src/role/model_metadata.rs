// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Model-specific metadata, intentionally empty in the first version.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelMetadata;
/// use qubit_model_metadata::metadata::ModelRole;
/// use qubit_model_metadata::metadata::RoleMetadata;
///
/// let model = ModelMetadata;
/// assert_eq!(RoleMetadata::Model(model).role(), ModelRole::Model);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelMetadata;
