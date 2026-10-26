// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain metadata anchored to one reflection descriptor root.

#[path = "type_metadata/enum_metadata.rs"]
mod enum_metadata;
#[path = "type_metadata/enum_variant_metadata.rs"]
mod enum_variant_metadata;
#[path = "type_metadata/has_type_metadata.rs"]
mod has_type_metadata;
#[path = "type_metadata/type_metadata.rs"]
#[allow(clippy::module_inception)]
mod type_metadata;

pub use self::enum_metadata::EnumMetadata;
pub use self::enum_variant_metadata::EnumVariantMetadata;
pub use self::has_type_metadata::HasTypeMetadata;
pub use self::type_metadata::TypeMetadata;
