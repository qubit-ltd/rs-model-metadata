// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! Selector and validator IR.

// qubit-style: allow multiple-public-types

use syn::LitStr;

use super::CodecIr;
use super::ConstraintIr;
use super::RedactIr;

#[derive(Clone)]
pub(crate) struct SelectorIr {
    pub(crate) position: SelectorPositionIr,
    pub(crate) constraints: Vec<ConstraintIr>,
    pub(crate) validators: Vec<ValidatorIr>,
    pub(crate) codec: Option<CodecIr>,
    pub(crate) redact: Option<RedactIr>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SelectorPositionIr {
    Element,
    MapKey,
    MapValue,
}

#[derive(Clone)]
pub(crate) enum StrategyArgumentIr {
    Bool(bool),
    Integer(i128),
    Unsigned(u128),
    String(LitStr),
    BoolList(Vec<bool>),
    IntegerList(Vec<i128>),
    UnsignedList(Vec<u128>),
    StringList(Vec<LitStr>),
}

#[derive(Clone)]
pub(crate) struct ValidatorIr {
    pub(crate) id: LitStr,
    pub(crate) params: Vec<(String, StrategyArgumentIr)>,
    pub(crate) depends_on: Vec<Vec<String>>,
    pub(crate) dependency_bindings: Vec<(String, Vec<String>)>,
    pub(crate) target: TargetModeIr,
    pub(crate) on_none: OnNoneIr,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum TargetModeIr {
    #[default]
    Value,
    Container,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum OnNoneIr {
    #[default]
    Skip,
    Reject,
}
