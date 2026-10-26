// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Sensitivity declared for redaction-aware consumers.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::Sensitivity;
///
/// assert_eq!(Sensitivity::Secret.as_str(), "secret");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Sensitivity {
    /// Information intended for unrestricted disclosure.
    Public,
    /// Personal information requiring ordinary access controls.
    Personal,
    /// Confidential information requiring restricted access.
    Confidential,
    /// Secret data.
    Secret,
}

impl Sensitivity {
    /// Returns the stable declaration spelling.
    ///
    /// # Returns
    ///
    /// The lowercase metadata spelling used by reflection consumers.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Personal => "personal",
            Self::Confidential => "confidential",
            Self::Secret => "secret",
        }
    }
}
