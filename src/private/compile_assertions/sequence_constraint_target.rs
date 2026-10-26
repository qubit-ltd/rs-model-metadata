// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Sequence element type selection for generated constraints.

use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

/// Exposes the element type selected by sequence constraints without relying
/// on a Rust type's source spelling.
///
/// This trait is implemented for slices, arrays, standard sequence
/// collections, `Option`, and transparent pointer wrappers around supported
/// shapes. Wrapper implementations recursively expose the inner element type,
/// which must be `'static` for generated selector metadata.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::v8::SequenceConstraintTarget;
/// use std::marker::PhantomData;
///
/// fn selected_element<T: SequenceConstraintTarget + ?Sized>(
///     _: &T,
/// ) -> PhantomData<T::Element> {
///     PhantomData
/// }
///
/// let values: &[u8] = &[1, 2, 3];
/// let _: PhantomData<u8> = selected_element(values);
/// ```
#[doc(hidden)]
pub trait SequenceConstraintTarget {
    /// Element type selected from the supported sequence shape.
    type Element: 'static;
}

/// Implements [`SequenceConstraintTarget`] for sequence containers whose
/// element type is known directly from the container type.
///
/// Each generated implementation exposes the contained `T` as its selected
/// element type and requires `T` to be `'static` for selector metadata.
macro_rules! sequence_target {
    ($($container:ty),+ $(,)?) => {
        $(impl<T: 'static> SequenceConstraintTarget for $container {
            type Element = T;
        })+
    };
}

sequence_target!(
    [T],
    Vec<T>,
    VecDeque<T>,
    LinkedList<T>,
    BinaryHeap<T>,
    HashSet<T>,
    BTreeSet<T>,
);

impl<T: 'static, const N: usize> SequenceConstraintTarget for [T; N] {
    type Element = T;
}

impl<T: SequenceConstraintTarget> SequenceConstraintTarget for Option<T> {
    type Element = T::Element;
}

/// Implements [`SequenceConstraintTarget`] for transparent pointer wrappers
/// around another supported sequence shape.
///
/// Each generated implementation forwards the selected element type from the
/// wrapped target, so the wrapper does not change which element constraints
/// select.
macro_rules! transparent_sequence_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: SequenceConstraintTarget + ?Sized> SequenceConstraintTarget for $wrapper {
            type Element = T::Element;
        })+
    };
}

transparent_sequence_target!(Box<T>, Rc<T>, Arc<T>);
