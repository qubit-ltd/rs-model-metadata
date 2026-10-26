// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Collects the source-side model declarations used by linked-workspace fixtures.

#[cfg(feature = "duplicate-fixture")]
mod duplicate;
#[cfg(feature = "missing-fixture")]
mod missing_target;
mod source;
mod target_view;

#[cfg(feature = "duplicate-fixture")]
pub use duplicate::Duplicate;
#[cfg(feature = "missing-fixture")]
pub use missing_target::MissingTarget;
pub use source::Source;
pub use target_view::TargetView;
