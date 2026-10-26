// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The five supported model roles and their role-specific payloads.

mod entity_metadata;
mod model_metadata;
mod model_role;
mod projection_metadata;
mod role_metadata;
mod value_metadata;

pub use entity_metadata::EntityMetadata;
pub use model_metadata::ModelMetadata;
pub use model_role::ModelRole;
pub use projection_metadata::ProjectionMetadata;
pub use role_metadata::RoleMetadata;
pub use value_metadata::ValueMetadata;
