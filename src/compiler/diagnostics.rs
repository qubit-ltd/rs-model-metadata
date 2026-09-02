// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregates independent macro diagnostics before emitting compiler errors.

/// Accumulates recoverable declaration diagnostics.
#[derive(Default)]
pub(crate) struct Diagnostics(Option<syn::Error>);

impl Diagnostics {
    /// Adds one diagnostic without discarding previously collected errors.
    pub(crate) fn push(&mut self, error: syn::Error) {
        if let Some(existing) = &mut self.0 {
            existing.combine(error);
        } else {
            self.0 = Some(error);
        }
    }

    /// Returns all accumulated diagnostics, if any.
    pub(crate) fn finish(self) -> syn::Result<()> {
        self.0.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::Error;

    use super::Diagnostics;

    #[test]
    fn test_finish_emits_all_combined_errors() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(Error::new(Span::call_site(), "first error"));
        diagnostics.push(Error::new(Span::call_site(), "second error"));

        let error = diagnostics.finish().expect_err("combined diagnostics must fail");
        let tokens = error.into_compile_error().to_string();

        assert!(tokens.contains("first error"));
        assert!(tokens.contains("second error"));
    }
}
