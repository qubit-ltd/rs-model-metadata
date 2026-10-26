// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Variable-length sequence shapes accepted by range constraints.

use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use super::SequenceConstraintTarget;

/// Marks sequence shapes whose length may be constrained by a range.
///
/// This marker is implemented for slices and standard-library variable-length
/// sequence containers, together with `Option` and transparent pointer wrappers
/// around another supported shape. Fixed-size arrays intentionally do not
/// implement this trait, so generated range constraints can distinguish them
/// from sequences whose length varies at runtime.
#[doc(hidden)]
pub trait VariableLengthSequenceTarget: SequenceConstraintTarget {}

macro_rules! variable_sequence_target {
    ($($container:ty),+ $(,)?) => {
        $(impl<T: 'static> VariableLengthSequenceTarget for $container {})+
    };
}

variable_sequence_target!(
    [T],
    Vec<T>,
    VecDeque<T>,
    LinkedList<T>,
    BinaryHeap<T>,
    HashSet<T>,
    BTreeSet<T>,
);

impl<T: VariableLengthSequenceTarget> VariableLengthSequenceTarget for Option<T> {}

macro_rules! transparent_variable_sequence_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: VariableLengthSequenceTarget + ?Sized> VariableLengthSequenceTarget for $wrapper {})+
    };
}

transparent_variable_sequence_target!(Box<T>, Rc<T>, Arc<T>);
