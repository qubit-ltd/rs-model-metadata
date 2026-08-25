// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata describing field paths and direct model relations.

mod field_path;
mod lookup_relation_metadata;
mod ownership_metadata;
mod reference_metadata;

pub use self::field_path::FieldPath;
pub use self::lookup_relation_metadata::LookupRelationMetadata;
pub use self::ownership_metadata::OwnershipMetadata;
pub use self::reference_metadata::ReferenceMetadata;
pub use self::reference_metadata::ReferencePath;
pub use self::reference_metadata::ReferencePathSegment;
pub use self::reference_metadata::ReferenceTarget;
