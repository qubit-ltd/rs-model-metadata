// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Codec binding diagnostics.

mod codec_bind_error;
mod codec_bind_error_kind;
mod codec_bind_errors;

pub use codec_bind_error::CodecBindError;
pub use codec_bind_error_kind::CodecBindErrorKind;
pub use codec_bind_errors::CodecBindErrors;
