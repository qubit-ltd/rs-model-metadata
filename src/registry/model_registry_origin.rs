// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Source coordinates retained for model registry entries.

use qubit_reflect::error::RegistrationOrigin;

/// Owned source coordinates retained from a vNext capability registration.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::registry::ModelRegistryOrigin;
///
/// let origin = ModelRegistryOrigin {
///     crate_name: "example_models",
///     crate_version: Some("1.0.0"),
///     module_path: "example_models::account",
///     file: Some("src/account.rs"),
///     line: Some(12),
///     column: Some(1),
/// };
/// assert_eq!(origin.crate_name, "example_models");
/// assert_eq!(origin.line, Some(12));
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModelRegistryOrigin {
    /// Crate containing the registration.
    pub crate_name: &'static str,
    /// Crate version recorded for the registration.
    pub crate_version: Option<&'static str>,
    /// Module containing the registration.
    pub module_path: &'static str,
    /// Source file, when available.
    pub file: Option<&'static str>,
    /// One-based source line, when available.
    pub line: Option<u32>,
    /// One-based source column, when available.
    pub column: Option<u32>,
}

impl From<&RegistrationOrigin> for ModelRegistryOrigin {
    /// Copies the registration's source coordinates into an owned value.
    ///
    /// # Returns
    ///
    /// A value containing the registration's crate, module, and source
    /// coordinates.
    #[inline]
    fn from(origin: &RegistrationOrigin) -> Self {
        Self {
            crate_name: origin.crate_name,
            crate_version: origin.crate_version,
            module_path: origin.module_path,
            file: origin.file,
            line: origin.line,
            column: origin.column,
        }
    }
}
