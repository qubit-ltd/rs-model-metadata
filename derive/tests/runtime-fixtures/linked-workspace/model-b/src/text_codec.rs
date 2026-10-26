// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Defines the cross-crate text codec and its static registration.

use core::convert::Infallible;

use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;

/// A policy implementation in a different crate from its model declaration.
#[derive(Default)]
pub struct TextCodec;

impl ValueEncoder<String> for TextCodec {
    /// The owned string produced by encoding.
    type Output = String;
    /// Encoding cannot fail for this identity codec.
    type Error = Infallible;

    /// Encodes a string by returning an owned copy without changing its text.
    ///
    /// # Parameters
    ///
    /// - `input`: String whose contents are copied into the encoded value.
    ///
    /// # Returns
    ///
    /// Returns an owned copy with unchanged contents.
    ///
    /// # Errors
    ///
    /// This implementation cannot fail because its error type is `Infallible`.
    fn encode(&mut self, input: &String) -> Result<String, Self::Error> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for TextCodec {
    /// The owned string produced by decoding.
    type Output = String;
    /// Decoding cannot fail for this identity codec.
    type Error = Infallible;

    /// Decodes text by returning an owned copy of the input.
    ///
    /// # Parameters
    ///
    /// - `input`: Text whose contents are copied into the decoded value.
    ///
    /// # Returns
    ///
    /// Returns an owned string with unchanged contents.
    ///
    /// # Errors
    ///
    /// This implementation cannot fail because its error type is `Infallible`.
    fn decode(&mut self, input: &str) -> Result<String, Self::Error> {
        Ok(input.to_owned())
    }
}

/// The immutable codec implementation metadata used by the root registration.
pub(super) static CODEC_DESCRIPTOR: ValueStringCodecDescriptor =
    ValueStringCodecDescriptor::of::<TextCodec, String>();
