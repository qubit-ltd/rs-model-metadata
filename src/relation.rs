// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static property paths used by declarations and resolver diagnostics.

mod declaration_location;
mod field_location;
mod field_path;
mod navigation_step;
mod object_path;
mod object_path_error;

pub use self::declaration_location::DeclarationLocation;
pub use self::field_location::FieldLocation;
pub use self::field_path::PropertyPath;
pub use self::navigation_step::NavigationStep;
pub use self::object_path::ObjectPath;
pub use self::object_path_error::ObjectPathError;
