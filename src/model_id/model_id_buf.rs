// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Owned stable model identifiers for dynamic inputs.

use core::borrow::Borrow;
use core::fmt;

use super::ModelId;
use super::ModelIdError;

/// A validated, owned stable identifier for a model type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelIdBuf(String);

impl ModelIdBuf {
    /// Returns the complete stable model ID.
    #[must_use]
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ModelIdBuf {
    type Error = ModelIdError;

    /// Validates and takes ownership of a dynamic model-ID string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ModelId::validate(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ModelIdBuf {
    type Error = ModelIdError;

    /// Validates and copies a dynamic model-ID string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ModelId> for ModelIdBuf {
    /// Copies a validated static model ID into an owned value.
    fn from(value: ModelId) -> Self {
        Self(value.as_str().to_owned())
    }
}

impl Borrow<str> for ModelIdBuf {
    /// Borrows the stable model ID as a string slice.
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ModelIdBuf {
    /// Returns the stable model ID as a string slice.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ModelIdBuf {
    /// Formats the stable model ID.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
