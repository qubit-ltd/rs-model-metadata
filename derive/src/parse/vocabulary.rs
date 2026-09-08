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
pub(crate) fn validate_closed_value(expression: &Expr, value: &str, allowed: &[&str], message: &str) -> Result<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(Error::new_spanned(expression, message))
    }
}

/// Validates the closed set of field redaction sensitivity levels.
pub(crate) fn validate_redact_level(level: &LitStr) -> Result<()> {
    if matches!(level.value().as_str(), "low" | "medium" | "high" | "secret") {
        Ok(())
    } else {
        Err(Error::new_spanned(
            level,
            "redact level must be low, medium, high, or secret",
        ))
    }
}
