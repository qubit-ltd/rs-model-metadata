// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Final Serde behavior for one field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SerdeBehaviorSource, SerdeFieldMetadata};
///
/// let serde = SerdeFieldMetadata::DEFAULT.with_sources(
///     SerdeBehaviorSource::Explicit,
///     SerdeBehaviorSource::Suppressed,
/// );
/// assert_eq!(serde.default_source(), SerdeBehaviorSource::Explicit);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerdeBehaviorSource {
    /// The behavior is absent.
    None,
    /// The behavior came from an explicit Serde field attribute.
    Explicit,
    /// The model macro supplied the behavior.
    ModelDefault,
    /// A model marker explicitly suppressed the model default.
    Suppressed,
}
