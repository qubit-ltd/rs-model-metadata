// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiled dynamic property access paths over one explicit model registry.

#[path = "property_access_path.rs"]
mod property_access_path;

pub use property_access_path::PropertyAccessPath;
pub use property_access_path::PropertyAccessPathError;
pub use property_access_path::PropertyAccessWriteFailure;
