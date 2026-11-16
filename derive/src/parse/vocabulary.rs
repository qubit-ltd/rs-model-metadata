// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validates closed metadata vocabularies while parsing attributes.

use syn::Error;
use syn::Expr;
use syn::LitStr;
use syn::Result;

/// Rejects one parsed identifier value outside its declared vocabulary.
///
/// # Parameters
/// - `expression`: The original syntax node used to locate a diagnostic.
/// - `value`: The parsed identifier value to check.
/// - `allowed`: The accepted values for this context.
/// - `message`: The diagnostic message used when `value` is not accepted.
///
/// # Returns
/// Returns `Ok(())` when `value` is in `allowed`.
///
/// # Errors
/// Returns a syntax error at `expression` when `value` is not in `allowed`;
/// the error uses `message` as its diagnostic text.
pub(crate) fn validate_closed_value(expression: &Expr, value: &str, allowed: &[&str], message: &str) -> Result<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(Error::new_spanned(expression, message))
    }
}

/// Validates the closed set of field redaction sensitivity levels.
///
/// # Parameters
/// - `level`: The string literal containing the requested sensitivity level.
///
/// # Returns
/// Returns `Ok(())` when the literal names a supported sensitivity level.
///
/// # Errors
/// Returns a syntax error located at `level` when its value is not one of the
/// supported levels.
pub(crate) fn validate_redact_level(level: &LitStr) -> Result<()> {
    if matches!(
        level.value().as_str(),
        "public" | "personal" | "confidential" | "secret" | "low" | "medium" | "high"
    ) {
        Ok(())
    } else {
        Err(Error::new_spanned(
            level,
            "redact level must be public, personal, confidential, secret, low, medium, or high",
        ))
    }
}
