// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A policy for handling sensitive values in downstream consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensitiveHandling {
    /// Redact the complete value.
    Redact,
    /// Mask part of the value while retaining contextual information.
    Mask,
    /// Apply the policy for authentication tokens and verification codes.
    Token,
}
