// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// Field, property, and role metadata validation are kept in separate units.
mod common;
mod fields;
mod properties;
mod roles;

pub(in crate::type_metadata) use common::abi_violation;
pub(in crate::type_metadata) use common::contains_field;
pub(in crate::type_metadata) use fields::descriptor_for;
pub(in crate::type_metadata) use fields::validate_fields;
pub(in crate::type_metadata) use properties::validate_properties;
pub(in crate::type_metadata) use roles::validate_role;
