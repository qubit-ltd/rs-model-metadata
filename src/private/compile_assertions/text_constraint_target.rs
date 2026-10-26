// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Text values accepted by generated constraints.

use std::rc::Rc;
use std::sync::Arc;

/// Marks types that generated text constraints can validate.
///
/// Implementations cover strings, optional values of supported types, and
/// pointer wrappers around supported targets.
#[doc(hidden)]
pub trait TextConstraintTarget {}

impl TextConstraintTarget for str {}
impl TextConstraintTarget for String {}
impl<T: TextConstraintTarget> TextConstraintTarget for Option<T> {}

/// Implements the marker trait for transparent pointer wrappers.
macro_rules! transparent_text_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: TextConstraintTarget + ?Sized> TextConstraintTarget for $wrapper {})+
    };
}

transparent_text_target!(Box<T>, Rc<T>, Arc<T>);
