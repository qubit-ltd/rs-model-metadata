// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Marker contract for generated `ModelImpl` implementations.

//! Generated code implements this trait once for each `ModelImpl` block to
//! identify the block without adding runtime data or callable behavior.

/// Identifies one generated `ModelImpl` block at the type level.
///
/// This zero-method marker is implemented by generated code; it adds no
/// runtime state or callable API. It is part of the hidden generated-code
/// interface and is not intended as a user customization point.
#[doc(hidden)]
pub trait ModelImplSeal {}
