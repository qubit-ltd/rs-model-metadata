// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Semantic values shared by the macro compiler stages.

pub(crate) mod declaration;
mod located;
mod macro_kind;

pub(crate) use located::Located;
pub(crate) use macro_kind::MacroKind;
