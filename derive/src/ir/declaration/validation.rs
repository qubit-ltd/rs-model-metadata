// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Selector and validator IR.

// qubit-style: allow multiple-public-types

use syn::LitStr;

use super::CodecIr;
use super::ConstraintIr;
use super::RedactIr;

#[derive(Clone)]
pub(crate) struct SelectorIr {
    /// Selected sequence or map position.
    pub(crate) position: SelectorPositionIr,
    /// Constraints declared for the selected values.
    pub(crate) constraints: Vec<ConstraintIr>,
    /// Validators declared for the selected values.
    pub(crate) validators: Vec<ValidatorIr>,
    /// Codec declared for the selected values.
    pub(crate) codec: Option<CodecIr>,
    /// Redaction declared for the selected values.
    pub(crate) redact: Option<RedactIr>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SelectorPositionIr {
    /// Sequence element position.
    Element,
    /// Map key position.
    MapKey,
    /// Map value position.
    MapValue,
}

#[derive(Clone)]
pub(crate) enum StrategyArgumentIr {
    /// Boolean argument.
    Bool(bool),
    /// Signed integer argument.
    Integer(i128),
    /// Unsigned integer argument.
    Unsigned(u128),
    /// String argument.
    String(LitStr),
    /// Boolean list argument.
    BoolList(Vec<bool>),
    /// Signed integer list argument.
    IntegerList(Vec<i128>),
    /// Unsigned integer list argument.
    UnsignedList(Vec<u128>),
    /// String list argument.
    StringList(Vec<LitStr>),
}

#[derive(Clone)]
pub(crate) struct ValidatorIr {
    /// Validator registry identifier.
    pub(crate) id: LitStr,
    /// Named validator arguments.
    pub(crate) params: Vec<(String, StrategyArgumentIr)>,
    /// Legacy dependency paths.
    pub(crate) depends_on: Vec<Vec<String>>,
    /// Named dependency bindings.
    pub(crate) dependency_bindings: Vec<(String, Vec<String>, Vec<String>)>,
}
