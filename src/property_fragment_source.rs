// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Source category for one unmerged model property declaration.

use crate::FieldMetadata;
use crate::GetterMetadata;
use crate::SetterMetadata;

/// Identifies the field or method that declared a property fragment.
#[derive(Clone, Copy, Debug)]
pub enum PropertyFragmentSource {
    /// A reflected storage field declared the property.
    Field(&'static FieldMetadata),
    /// A public getter declared a readable property fragment.
    Getter(&'static GetterMetadata),
    /// A public setter declared a writable property fragment.
    Setter(&'static SetterMetadata),
}
