// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Sequence shapes for which uniqueness constraints are meaningful.

use std::collections::BinaryHeap;
use std::collections::LinkedList;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use super::SequenceConstraintTarget;

/// Marks sequence shapes for which `unique_items` is not redundant.
#[doc(hidden)]
pub trait UniqueItemsConstraintTarget: SequenceConstraintTarget {}

/// Implements the uniqueness marker for supported sequence containers.
///
/// Each supplied container shape receives an implementation when its element
/// type is `'static`, matching the bound required by generated selector
/// metadata.
macro_rules! unique_items_target {
    ($($container:ty),+ $(,)?) => {
        $(impl<T: 'static> UniqueItemsConstraintTarget for $container {})+
    };
}

unique_items_target!([T], Vec<T>, VecDeque<T>, LinkedList<T>, BinaryHeap<T>,);

impl<T: 'static, const N: usize> UniqueItemsConstraintTarget for [T; N] {}
impl<T: UniqueItemsConstraintTarget> UniqueItemsConstraintTarget for Option<T> {}

/// Propagates the uniqueness marker through transparent pointer wrappers.
///
/// A wrapper qualifies only when its possibly unsized inner type already
/// implements [`UniqueItemsConstraintTarget`], so wrapping a scalar does not
/// make it a supported sequence.
macro_rules! transparent_unique_items_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: UniqueItemsConstraintTarget + ?Sized> UniqueItemsConstraintTarget for $wrapper {})+
    };
}

transparent_unique_items_target!(Box<T>, Rc<T>, Arc<T>);
