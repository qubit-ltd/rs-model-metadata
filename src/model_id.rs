// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable, portable identifiers for model types.

use core::borrow::Borrow;

mod model_id_buf;
mod model_id_error;

pub use self::model_id_buf::ModelIdBuf;
pub use self::model_id_error::ModelIdError;

/// A stable identifier for a model type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(&'static str);

impl ModelId {
    /// Validates a model-ID string without allocating.
    ///
    /// # Errors
    ///
    /// Returns [`ModelIdError`] when `value` does not follow the stable-ID
    /// protocol.
    #[inline]
    pub const fn validate(value: &str) -> Result<(), ModelIdError> {
        validate_model_id(value)
    }

    /// Creates a validated model ID from a static value.
    ///
    /// # Panics
    ///
    /// Panics when `value` does not follow the stable-ID protocol. Use
    /// [`ModelId::try_new`] when callers need a recoverable error.
    #[must_use]
    #[inline(always)]
    pub const fn new(value: &'static str) -> Self {
        match Self::validate(value) {
            Ok(()) => Self(value),
            Err(_) => panic!("invalid model ID"),
        }
    }

    /// Validates and creates a model ID from a static value.
    ///
    /// Returns [`ModelIdError`] when `value` does not follow the stable-ID
    /// protocol.
    #[inline]
    pub fn try_new(value: &'static str) -> Result<Self, ModelIdError> {
        Self::validate(value)?;
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
const fn validate_model_id(value: &str) -> Result<(), ModelIdError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(ModelIdError::Empty);
    }
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            if start == index || index + 1 == bytes.len() {
                return Err(ModelIdError::EmptySegment);
            }
            if let Err(error) = validate_module_segment(bytes, start, index) {
                return Err(error);
            }
            start = index + 1;
        }
        index += 1;
    }
    if start == bytes.len() {
        return Err(ModelIdError::EmptySegment);
    }
    validate_type_segment(bytes, start, bytes.len())
}

/// Validates one ASCII snake-case module segment.
const fn validate_module_segment(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<(), ModelIdError> {
    if is_rust_keyword(bytes, start, end) {
        return Err(ModelIdError::KeywordModuleSegment);
    }
    if !(bytes[start] >= b'a' && bytes[start] <= b'z') || bytes[end - 1] == b'_'
    {
        return Err(ModelIdError::InvalidModuleSegment);
    }
    let mut previous_underscore = false;
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !((byte >= b'a' && byte <= b'z')
            || (byte >= b'0' && byte <= b'9')
            || byte == b'_')
            || (byte == b'_' && previous_underscore)
        {
            return Err(ModelIdError::InvalidModuleSegment);
        }
        previous_underscore = byte == b'_';
        index += 1;
    }
    Ok(())
}

/// Validates the final ASCII UpperCamelCase type segment.
const fn validate_type_segment(
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<(), ModelIdError> {
    if !(bytes[start] >= b'A' && bytes[start] <= b'Z') {
        return Err(ModelIdError::InvalidTypeSegment);
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !((byte >= b'a' && byte <= b'z')
            || (byte >= b'A' && byte <= b'Z')
            || (byte >= b'0' && byte <= b'9'))
        {
            return Err(ModelIdError::InvalidTypeSegment);
        }
        index += 1;
    }
    Ok(())
}

/// Returns whether a segment is reserved by Rust 2024.
const fn is_rust_keyword(bytes: &[u8], start: usize, end: usize) -> bool {
    segment_equals(bytes, start, end, b"as")
        || segment_equals(bytes, start, end, b"async")
        || segment_equals(bytes, start, end, b"await")
        || segment_equals(bytes, start, end, b"break")
        || segment_equals(bytes, start, end, b"const")
        || segment_equals(bytes, start, end, b"continue")
        || segment_equals(bytes, start, end, b"crate")
        || segment_equals(bytes, start, end, b"else")
        || segment_equals(bytes, start, end, b"enum")
        || segment_equals(bytes, start, end, b"extern")
        || segment_equals(bytes, start, end, b"false")
        || segment_equals(bytes, start, end, b"fn")
        || segment_equals(bytes, start, end, b"for")
        || segment_equals(bytes, start, end, b"gen")
        || segment_equals(bytes, start, end, b"if")
        || segment_equals(bytes, start, end, b"impl")
        || segment_equals(bytes, start, end, b"in")
        || segment_equals(bytes, start, end, b"let")
        || segment_equals(bytes, start, end, b"loop")
        || segment_equals(bytes, start, end, b"match")
        || segment_equals(bytes, start, end, b"mod")
        || segment_equals(bytes, start, end, b"move")
        || segment_equals(bytes, start, end, b"mut")
        || segment_equals(bytes, start, end, b"pub")
        || segment_equals(bytes, start, end, b"ref")
        || segment_equals(bytes, start, end, b"return")
        || segment_equals(bytes, start, end, b"self")
        || segment_equals(bytes, start, end, b"Self")
        || segment_equals(bytes, start, end, b"static")
        || segment_equals(bytes, start, end, b"struct")
        || segment_equals(bytes, start, end, b"super")
        || segment_equals(bytes, start, end, b"trait")
        || segment_equals(bytes, start, end, b"true")
        || segment_equals(bytes, start, end, b"type")
        || segment_equals(bytes, start, end, b"unsafe")
        || segment_equals(bytes, start, end, b"use")
        || segment_equals(bytes, start, end, b"where")
        || segment_equals(bytes, start, end, b"while")
        || segment_equals(bytes, start, end, b"abstract")
        || segment_equals(bytes, start, end, b"become")
        || segment_equals(bytes, start, end, b"box")
        || segment_equals(bytes, start, end, b"do")
        || segment_equals(bytes, start, end, b"final")
        || segment_equals(bytes, start, end, b"macro")
        || segment_equals(bytes, start, end, b"override")
        || segment_equals(bytes, start, end, b"priv")
        || segment_equals(bytes, start, end, b"typeof")
        || segment_equals(bytes, start, end, b"unsized")
        || segment_equals(bytes, start, end, b"virtual")
        || segment_equals(bytes, start, end, b"yield")
}

/// Returns whether a byte range equals a static keyword.
const fn segment_equals(
    bytes: &[u8],
    start: usize,
    end: usize,
    keyword: &[u8],
) -> bool {
    if end - start != keyword.len() {
        return false;
    }
    let mut index = 0;
    while index < keyword.len() {
        if bytes[start + index] != keyword[index] {
            return false;
        }
        index += 1;
    }
    true
}
