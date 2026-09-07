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

//! Declaration-level options.

use syn::{LitStr, Type};

#[derive(Clone)]
pub(crate) struct DeclarationOptions {
    pub(crate) id: Option<LitStr>,
    pub(crate) source: Option<Type>,
    pub(crate) source_id: Option<LitStr>,
    pub(crate) open: bool,
    pub(crate) transparent: bool,
    pub(crate) no_clone: bool,
    pub(crate) no_debug: bool,
    pub(crate) no_display: bool,
    pub(crate) no_partial_eq: bool,
    pub(crate) no_eq: bool,
    pub(crate) no_hash: bool,
    pub(crate) no_serialize: bool,
    pub(crate) no_deserialize: bool,
    pub(crate) no_redact: bool,
    pub(crate) no_copy: bool,
    pub(crate) copy: bool,
    pub(crate) default: bool,
    pub(crate) partial_ord: bool,
    pub(crate) ord: bool,
    pub(crate) codec: Option<Type>,
}
