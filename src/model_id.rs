// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable, portable identifiers for model types.

use core::borrow::Borrow;

mod model_id_error;

pub use self::model_id_error::ModelIdError;

/// A stable identifier for a model type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(&'static str);

impl ModelId {
    /// Creates a model ID from a trusted static value.
    ///
    /// The caller must validate `value` before calling this function. Use
    /// [`ModelId::try_from_static`] for untrusted values.
    #[must_use]
    #[inline(always)]
    pub const fn from_static(value: &'static str) -> Self {
        Self(value)
    }

    /// Validates and creates a model ID from a static value.
    ///
    /// Returns [`ModelIdError`] when `value` does not follow the stable-ID
    /// protocol.
    #[inline]
    pub fn try_from_static(value: &'static str) -> Result<Self, ModelIdError> {
        validate_model_id(value)?;
        Ok(Self(value))
    }

    /// Returns the complete stable model ID.
    #[must_use]
    #[inline(always)]
    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Returns the final type-name segment of this model ID.
    #[must_use]
    #[inline]
    pub const fn type_name(self) -> &'static str {
        let bytes = self.0.as_bytes();
        let mut index = bytes.len();
        while index > 0 {
            index -= 1;
            if bytes[index] == b'.' {
                let (_, type_name) = self.0.split_at(index + 1);
                return type_name;
            }
        }
        self.0
    }
}

impl Borrow<str> for ModelId {
    /// Borrows the stable model ID as a string slice.
    fn borrow(&self) -> &str {
        self.0
    }
}

/// Validates a stable model-ID string without allocating.
fn validate_model_id(value: &str) -> Result<(), ModelIdError> {
    if value.is_empty() {
        return Err(ModelIdError::Empty);
    }
    if value.split('.').any(str::is_empty) {
        return Err(ModelIdError::EmptySegment);
    }
    let mut segments = value.split('.').peekable();
    while let Some(segment) = segments.next() {
        if segments.peek().is_some() {
            validate_module_segment(segment)?;
        } else {
            validate_type_segment(segment)?;
        }
    }
    Ok(())
}

/// Validates one ASCII snake-case module segment.
fn validate_module_segment(segment: &str) -> Result<(), ModelIdError> {
    if is_rust_keyword(segment) {
        return Err(ModelIdError::KeywordModuleSegment);
    }
    let bytes = segment.as_bytes();
    if !matches!(bytes.first(), Some(b'a'..=b'z'))
        || matches!(bytes.last(), Some(b'_'))
    {
        return Err(ModelIdError::InvalidModuleSegment);
    }
    let mut previous_underscore = false;
    for &byte in &bytes[1..] {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || (byte == b'_' && previous_underscore)
        {
            return Err(ModelIdError::InvalidModuleSegment);
        }
        previous_underscore = byte == b'_';
    }
    Ok(())
}

/// Validates the final ASCII UpperCamelCase type segment.
fn validate_type_segment(segment: &str) -> Result<(), ModelIdError> {
    let bytes = segment.as_bytes();
    if !matches!(bytes.first(), Some(b'A'..=b'Z'))
        || !bytes[1..].iter().all(u8::is_ascii_alphanumeric)
    {
        return Err(ModelIdError::InvalidTypeSegment);
    }
    Ok(())
}

/// Returns whether a segment is reserved by Rust 2024.
fn is_rust_keyword(segment: &str) -> bool {
    matches!(
        segment,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}
