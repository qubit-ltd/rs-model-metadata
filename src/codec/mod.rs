// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Binding of declared codec occurrences to executable registrations.

mod binder;
mod error;

pub use binder::CodecBindInputs;
pub use binder::CodecBinding;
pub use binder::CodecBindings;
pub use binder::CodecOccurrenceId;
pub use binder::bind_codecs;
pub use error::CodecBindError;
pub use error::CodecBindErrorKind;
pub use error::CodecBindErrors;
