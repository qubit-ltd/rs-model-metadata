// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for declaration parser output.

use syn::DataEnum;
use syn::ItemEnum;
use syn::parse_quote;

use crate::parse::declaration::parse_variants;

#[test]
fn test_enum_field_ir_retains_its_variant_index() {
    let item: ItemEnum = parse_quote! {
        enum Example {
            First { value: u8 },
            Second(u16),
        }
    };
    let data = DataEnum {
        enum_token: item.enum_token,
        brace_token: item.brace_token,
        variants: item.variants,
    };
    let variants = parse_variants(&data).expect("enum fields should parse");
    assert_eq!(variants[0].fields[0].variant_index, Some(0));
    assert_eq!(variants[1].fields[0].variant_index, Some(1));
}
