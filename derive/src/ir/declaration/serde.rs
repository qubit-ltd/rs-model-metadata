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

//! Serde overlay IR.

// qubit-style: allow public-type-layout

use syn::LitStr;

#[derive(Clone, Default)]
pub(crate) struct SerdeIr {
    pub(crate) serialize_name: Option<LitStr>,
    pub(crate) deserialize_name: Option<LitStr>,
    pub(crate) skip_serializing: bool,
    pub(crate) skip_deserializing: bool,
    pub(crate) flatten: bool,
    pub(crate) with: Option<LitStr>,
    pub(crate) default: bool,
    pub(crate) explicit_skip_serializing_if: bool,
    pub(crate) default_from_model: bool,
    pub(crate) omit_from_model: bool,
    pub(crate) omit_suppressed: bool,
}
