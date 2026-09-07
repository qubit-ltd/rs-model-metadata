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

//! Relationship and uniqueness IR.

use syn::{LitStr, Type};

#[derive(Clone)]
pub(crate) struct UniqueIr {
    pub(crate) respect_to: Vec<Vec<String>>,
    pub(crate) ignore_case: bool,
}

#[derive(Clone)]
pub(crate) enum ReferenceTargetIr {
    RustType(Box<Type>),
    ModelId(LitStr),
}

#[derive(Clone)]
pub(crate) struct ReferenceIr {
    pub(crate) target: ReferenceTargetIr,
    pub(crate) property: Option<Vec<String>>,
    pub(crate) existing: bool,
    pub(crate) same_as: Option<Vec<String>>,
}
