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

//! Field and enum-variant IR.

// qubit-style: allow multiple-public-types

use syn::Type;

use super::super::Located;
use super::CodecIr;
use super::ConstraintIr;
use super::RedactIr;
use super::ReferenceIr;
use super::SelectorIr;
use super::SerdeIr;
use super::ValidatorIr;

#[derive(Clone)]
pub(crate) enum FieldOccurrence {
    Identifier(IdentifierAssignmentIr),
    Indexed,
    Unique(super::UniqueIr),
    Reference(ReferenceIr),
    KeyPart(usize),
    Constraint(ConstraintIr),
    Selector(SelectorIr),
    Validator(ValidatorIr),
    Codec(CodecIr),
    Redact(RedactIr),
    Serde(SerdeIr),
    Opaque,
    ValidateNested,
}

#[derive(Clone, Copy)]
pub(crate) enum IdentifierAssignmentIr {
    Application,
    Database,
}

#[derive(Clone)]
pub(crate) struct FieldIr {
    pub(crate) index: Located<usize>,
    pub(crate) ty: Type,
    pub(crate) occurrences: Vec<FieldOccurrence>,
    pub(crate) keep_serializing: bool,
    pub(crate) named: bool,
}

#[derive(Clone)]
pub(crate) struct VariantIr {
    pub(crate) rust_name: String,
    pub(crate) canonical_name: String,
    pub(crate) serialized_name: String,
    pub(crate) deserialized_name: String,
    pub(crate) default: bool,
    pub(crate) fields: Vec<FieldIr>,
}
