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

//! Normalized validation constraints.

// qubit-style: allow multiple-public-types

use syn::LitStr;

#[derive(Clone)]
pub(crate) enum ConstraintIr {
    Text(TextConstraintIr),
    Decimal(DecimalConstraintIr),
    Time(String),
    Sequence {
        min: Option<usize>,
        max: Option<usize>,
        unique: bool,
    },
    Map {
        min: Option<usize>,
        max: Option<usize>,
    },
}

#[derive(Clone, Default)]
pub(crate) struct TextConstraintIr {
    pub(crate) min_chars: Option<u32>,
    pub(crate) max_chars: Option<u32>,
    pub(crate) min_bytes: Option<u32>,
    pub(crate) max_bytes: Option<u32>,
    pub(crate) allowed_chars: Option<String>,
    pub(crate) non_blank: bool,
    pub(crate) format: Option<String>,
}

#[derive(Clone)]
pub(crate) struct DecimalConstraintIr {
    pub(crate) precision: Option<u16>,
    pub(crate) scale: u16,
    pub(crate) rounding: String,
    pub(crate) money: bool,
    pub(crate) min: Option<LitStr>,
    pub(crate) max: Option<LitStr>,
    pub(crate) min_inclusive: bool,
    pub(crate) max_inclusive: bool,
}
