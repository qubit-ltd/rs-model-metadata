// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::serde_behavior_source::SerdeBehaviorSource;

/// Final Serde behavior for one field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::SerdeFieldMetadata;
///
/// let serde = SerdeFieldMetadata::DEFAULT;
/// assert_eq!(serde.serialize_name(), None);
/// assert!(!serde.skip_serializing());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerdeFieldMetadata {
    /// The explicit serialization name, when one is configured.
    serialize_name: Option<&'static str>,
    /// The explicit deserialization name, when one is configured.
    deserialize_name: Option<&'static str>,
    /// Whether Serde omits this field while serializing.
    skip_serializing: bool,
    /// Whether Serde ignores this field while deserializing.
    skip_deserializing: bool,
    /// Whether Serde flattens this field into its parent representation.
    flatten: bool,
    /// The Serde conversion module or function path, when configured.
    with: Option<&'static str>,
    /// Whether Serde supplies a missing value through its default behavior.
    default: bool,
    /// The origin of the missing-value default behavior.
    default_source: SerdeBehaviorSource,
    /// The origin of empty-value omission behavior.
    omit_source: SerdeBehaviorSource,
}

impl SerdeFieldMetadata {
    /// Empty Serde behavior used when no configuration applies.
    pub const DEFAULT: Self = Self::new(None, None, false, false, false, None, false);

    /// Creates final Serde field behavior.
    ///
    /// # Parameters
    ///
    /// * `serialize_name` - Optional serialized field name.
    /// * `deserialize_name` - Optional deserialized field name.
    /// * `skip_serializing` - Whether serialization omits the field.
    /// * `skip_deserializing` - Whether deserialization ignores the field.
    /// * `flatten` - Whether Serde merges this field into its parent.
    /// * `with` - Optional Serde conversion path.
    /// * `default` - Whether missing values use Serde default behavior.
    ///
    /// # Returns
    ///
    /// Effective field behavior with provenance inferred from explicit flags.
    #[must_use]
    #[inline]
    pub const fn new(
        serialize_name: Option<&'static str>,
        deserialize_name: Option<&'static str>,
        skip_serializing: bool,
        skip_deserializing: bool,
        flatten: bool,
        with: Option<&'static str>,
        default: bool,
    ) -> Self {
        Self {
            serialize_name,
            deserialize_name,
            skip_serializing,
            skip_deserializing,
            flatten,
            with,
            default,
            default_source: if default {
                SerdeBehaviorSource::Explicit
            } else {
                SerdeBehaviorSource::None
            },
            omit_source: SerdeBehaviorSource::None,
        }
    }

    /// Returns the configured serialization name, or `None` when Serde uses
    /// the field name.
    ///
    /// # Returns
    ///
    /// The serialization name override, or `None` to retain the field name.
    #[inline]
    pub const fn serialize_name(&self) -> Option<&'static str> {
        self.serialize_name
    }

    /// Returns the configured deserialization name, or `None` when Serde uses
    /// the field name.
    ///
    /// # Returns
    ///
    /// The deserialization name override, or `None` to retain the field name.
    #[inline]
    pub const fn deserialize_name(&self) -> Option<&'static str> {
        self.deserialize_name
    }

    /// Returns whether Serde omits this field during serialization.
    ///
    /// # Returns
    ///
    /// `true` when serialization skips this field.
    #[must_use]
    #[inline]
    pub const fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }

    /// Returns whether Serde ignores this field during deserialization.
    ///
    /// # Returns
    ///
    /// `true` when deserialization skips this field.
    #[must_use]
    #[inline]
    pub const fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }

    /// Returns whether Serde flattens this field into its parent.
    ///
    /// # Returns
    ///
    /// `true` when this field is merged into its parent representation.
    #[must_use]
    #[inline]
    pub const fn flatten(&self) -> bool {
        self.flatten
    }

    /// Returns the configured Serde conversion path, if present.
    ///
    /// # Returns
    ///
    /// The configured conversion path, or `None` when absent.
    #[inline]
    pub const fn with(&self) -> Option<&'static str> {
        self.with
    }

    /// Returns whether a missing value uses Serde's default behavior.
    ///
    /// # Returns
    ///
    /// `true` when missing values are supplied by default behavior.
    #[must_use]
    #[inline]
    pub const fn default(&self) -> bool {
        self.default
    }

    /// Returns the declaration source for missing-value defaults.
    ///
    /// # Returns
    ///
    /// The origin of the missing-value default behavior.
    #[must_use]
    #[inline]
    pub const fn default_source(&self) -> SerdeBehaviorSource {
        self.default_source
    }

    /// Returns the declaration source for empty-value omission.
    ///
    /// # Returns
    ///
    /// The origin of empty-value omission behavior.
    #[must_use]
    #[inline]
    pub const fn omit_source(&self) -> SerdeBehaviorSource {
        self.omit_source
    }

    /// Records whether missing-value defaults and empty-value omission were
    /// explicit, generated, or suppressed.
    ///
    /// # Parameters
    ///
    /// * `default_source` - Provenance of missing-value default behavior.
    /// * `omit_source` - Provenance of empty-value omission behavior.
    ///
    /// # Returns
    ///
    /// This metadata with both behavior provenance values recorded.
    #[must_use]
    #[inline]
    pub const fn with_sources(mut self, default_source: SerdeBehaviorSource, omit_source: SerdeBehaviorSource) -> Self {
        self.default_source = default_source;
        self.omit_source = omit_source;
        self
    }
}
