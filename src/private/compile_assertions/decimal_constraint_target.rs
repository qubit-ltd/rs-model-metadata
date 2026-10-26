// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Exact decimal values accepted by generated constraints.

use std::rc::Rc;
use std::sync::Arc;

use bigdecimal::BigDecimal;

/// Marks exact decimal values accepted by decimal and money constraints.
#[doc(hidden)]
pub trait DecimalConstraintTarget {}

macro_rules! decimal_target {
    ($($type:ty),+ $(,)?) => {
        $(impl DecimalConstraintTarget for $type {})+
    };
}

decimal_target!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, BigDecimal
);

impl<T: DecimalConstraintTarget> DecimalConstraintTarget for Option<T> {}

macro_rules! transparent_decimal_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: DecimalConstraintTarget + ?Sized> DecimalConstraintTarget for $wrapper {})+
    };
}

transparent_decimal_target!(Box<T>, Rc<T>, Arc<T>);
