// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Temporal values accepted by generated constraints.

use std::rc::Rc;
use std::sync::Arc;

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::TimeZone;

/// Marks temporal values accepted by time constraints.
#[doc(hidden)]
pub trait TemporalConstraintTarget {}

impl<Tz: TimeZone + 'static> TemporalConstraintTarget for DateTime<Tz> {}
impl TemporalConstraintTarget for NaiveDate {}
impl TemporalConstraintTarget for NaiveDateTime {}
impl TemporalConstraintTarget for NaiveTime {}
impl<T: TemporalConstraintTarget> TemporalConstraintTarget for Option<T> {}

/// Implements the temporal target marker for pointer-like wrappers.
///
/// Each generated implementation preserves the wrapped type's eligibility as
/// a temporal constraint target, including for unsized wrapped types. This is
/// a compile-time marker implementation and does not add runtime conversion or
/// dereferencing behavior.
macro_rules! transparent_temporal_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: TemporalConstraintTarget + ?Sized> TemporalConstraintTarget for $wrapper {})+
    };
}

transparent_temporal_target!(Box<T>, Rc<T>, Arc<T>);
