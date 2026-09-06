// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates field metadata, constraints, validators, codecs, and Serde
//! overlays.

use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::LitStr;

use super::role::codec_value_type;
use crate::ir::declaration::CodecIr;
use crate::ir::declaration::ConstraintIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::IdentifierAssignmentIr;
use crate::ir::declaration::OnNoneIr;
use crate::ir::declaration::RedactIr;
use crate::ir::declaration::RedactModeIr;
use crate::ir::declaration::ReferenceIr;
use crate::ir::declaration::ReferenceTargetIr;
use crate::ir::declaration::SelectorIr;
use crate::ir::declaration::SelectorPositionIr;
use crate::ir::declaration::SerdeIr;
use crate::ir::declaration::StrategyArgumentIr;
use crate::ir::declaration::TargetModeIr;
use crate::ir::declaration::ValidatorIr;

/// Generates the runtime field vector for a declaration.
pub(super) fn expand_field_vector(
    fields: &[FieldIr],
    descriptor_fields: TokenStream,
    runtime: &TokenStream,
) -> TokenStream {
    let bodies = fields
        .iter()
        .map(|field| expand_field(field, &descriptor_fields, runtime, None));
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

/// Generates field overlays for a source-level generic declaration.
pub(super) fn expand_generic_field_vector(
    fields: &[FieldIr],
    descriptor_fields: TokenStream,
    runtime: &TokenStream,
    variant_inherited: bool,
) -> TokenStream {
    let bodies = fields
        .iter()
        .map(|field| expand_field(field, &descriptor_fields, runtime, Some(variant_inherited)));
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

/// Generates one field descriptor and its normalized attributes.
fn expand_field(
    field: &FieldIr,
    descriptor_fields: &TokenStream,
    runtime: &TokenStream,
    generic_variant_inherited: Option<bool>,
) -> TokenStream {
    let index = *field.index.value();
    let field_type = &field.ty;
    let validator_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Validator(value) => Some(value),
            _ => None,
        })
        .collect();
    let validators = validator_irs.iter().map(|value| expand_validator(value, runtime));
    let identifier_assignment = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Identifier(value) => Some(*value),
        _ => None,
    });
    let has_identifier = identifier_assignment.is_some();
    let has_indexed = field
        .occurrences
        .iter()
        .any(|value| matches!(value, FieldOccurrence::Indexed));
    let unique_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Unique(value) => Some(value),
        _ => None,
    });
    let reference_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Reference(value) => Some(value),
        _ => None,
    });
    let key_part_order = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::KeyPart(value) => Some(*value),
        _ => None,
    });
    let codec_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Codec(value) => Some(value),
        _ => None,
    });
    let redact_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Redact(value) => Some(value),
        _ => None,
    });
    let serde_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Serde(value) => Some(value),
        _ => None,
    });
    let element_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::Element) => Some(value),
        _ => None,
    });
    let map_key_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapKey) => Some(value),
        _ => None,
    });
    let map_value_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapValue) => Some(value),
        _ => None,
    });
    let element_selector = element_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v4::SequenceConstraintTarget>::Element
        );
        expand_selector_metadata(value, &value_type, format_ident!("element_selector"), runtime)
    });
    let map_key_selector = map_key_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v4::MapConstraintTarget>::Key
        );
        expand_selector_metadata(value, &value_type, format_ident!("map_key_selector"), runtime)
    });
    let map_value_selector = map_value_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v4::MapConstraintTarget>::Value
        );
        expand_selector_metadata(value, &value_type, format_ident!("map_value_selector"), runtime)
    });
    let constraint_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Constraint(value) => Some(value),
            _ => None,
        })
        .collect();
    let constraints = constraint_irs.iter().map(|value| {
        expand_constraint(
            value,
            element_ir.is_some(),
            map_key_ir.is_some(),
            map_value_ir.is_some(),
            runtime,
        )
    });
    let constraint_assertions = expand_constraint_assertions(&constraint_irs, quote!(#field_type), runtime);
    let requires_sequence = element_ir.is_some()
        || constraint_irs
            .iter()
            .any(|value| matches!(value, ConstraintIr::Sequence { .. }));
    let requires_map = map_key_ir.is_some()
        || map_value_ir.is_some()
        || constraint_irs
            .iter()
            .any(|value| matches!(value, ConstraintIr::Map { .. }));
    let sequence_assertion = requires_sequence.then(|| {
        quote! {
            fn assert_sequence_target<T: #runtime::__private::v4::SequenceConstraintTarget>() {}
            assert_sequence_target::<#field_type>();
        }
    });
    let map_assertion = requires_map.then(|| {
        quote! {
            fn assert_map_target<T: #runtime::__private::v4::MapConstraintTarget>() {}
            assert_map_target::<#field_type>();
        }
    });
    let identifier = identifier_assignment.map(|assignment| {
        let assignment = match assignment {
            IdentifierAssignmentIr::Application => {
                quote!(#runtime::IdentifierAssignment::Application)
            }
            IdentifierAssignmentIr::Database => quote!(#runtime::IdentifierAssignment::Database),
        };
        quote! {
            fn assert_identifier_type<T: #runtime::__private::v4::IdentifierType>() {}
            assert_identifier_type::<#field_type>();
            let identifier: &'static #runtime::IdentifierMetadata = #runtime::__private::v4::leak(
                #runtime::IdentifierMetadata::new(#assignment),
            );
        }
    });
    let unique = unique_ir.map(|unique| {
        let paths = unique.respect_to.iter().map(|path| expand_field_path(path, runtime));
        let ignore_case = unique.ignore_case;
        quote! {
            let unique_paths: &'static [#runtime::PropertyPath] = #runtime::__private::v4::leak_slice(::std::vec![#(#paths),*]);
            let unique: &'static #runtime::FieldUniqueMetadata = #runtime::__private::v4::leak(
                #runtime::FieldUniqueMetadata::new(unique_paths, #ignore_case),
            );
        }
    });
    let reference = reference_ir.map(|value| expand_reference(value, runtime));
    let key_part = key_part_order.map(|order| {
        quote! {
            let key_part: &'static #runtime::KeyPartMetadata = #runtime::__private::v4::leak(
                #runtime::KeyPartMetadata::new(#order),
            );
        }
    });
    let codec = codec_ir.map(|codec| {
        let value = match codec {
            CodecIr::DeclaredId(id) => quote!(#runtime::CodecReference::DeclaredId(#id)),
            CodecIr::RustType(ty) => {
                let value_type = codec_value_type(&field.ty);
                quote!(#runtime::CodecReference::RustType(#runtime::__private::v4::leak(
                    #runtime::__private::v4::ValueCodecDescriptor::of::<#ty, #value_type>(),
                )))
            }
        };
        quote! {
            let codec_reference: &'static #runtime::CodecReference = #runtime::__private::v4::leak(#value);
            let codec: &'static #runtime::CodecMetadata = #runtime::__private::v4::leak(
                #runtime::CodecMetadata::new(codec_reference, #runtime::CodecSource::Field),
            );
        }
    });
    let redact = redact_ir.map(|value| expand_redact(value, quote!(#runtime::RedactPosition::Field), runtime));
    let serde = serde_ir.map_or_else(
        || quote!(let serde: &'static #runtime::SerdeFieldMetadata = &#runtime::SerdeFieldMetadata::DEFAULT;),
        |value| expand_serde(value, runtime),
    );
    let mut occurrence_tokens = Vec::new();
    let mut validator_index = 0usize;
    let mut constraint_index = 0usize;
    for occurrence in &field.occurrences {
        occurrence_tokens.push(match occurrence {
            FieldOccurrence::Identifier(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Identifier(identifier));)
            }
            FieldOccurrence::Indexed => TokenStream::new(),
            FieldOccurrence::Unique(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Unique(unique));),
            FieldOccurrence::Reference(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Reference(reference));)
            }
            FieldOccurrence::KeyPart(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::KeyPart(key_part));)
            }
            FieldOccurrence::Constraint(_) => {
                let current = constraint_index;
                constraint_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Constraint(&constraints[#current]));)
            }
            FieldOccurrence::Selector(_) => TokenStream::new(),
            FieldOccurrence::Validator(_) => {
                let current = validator_index;
                validator_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Validator(&validators[#current]));)
            }
            FieldOccurrence::Codec(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Codec(codec));),
            FieldOccurrence::Redact(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Redact(redact));),
            FieldOccurrence::Serde(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Serde(serde));),
            FieldOccurrence::Opaque => quote!(attributes.push(#runtime::FieldAttributeMetadata::Opaque);),
        });
    }
    let mut reason_parts = Vec::new();
    if has_indexed {
        reason_parts.push(quote!(#runtime::IndexingReasons::EXPLICIT));
    }
    if has_identifier {
        reason_parts.push(quote!(#runtime::IndexingReasons::IDENTIFIER));
    }
    if unique_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::UNIQUE));
    }
    if reference_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::REFERENCE));
    }
    let indexed = reason_parts
        .into_iter()
        .reduce(|left, right| quote!(#left | #right))
        .map(|reasons| {
            quote! {
                attributes.push(#runtime::FieldAttributeMetadata::Indexed(#reasons));
            }
        });
    let metadata = match generic_variant_inherited {
        Some(variant_inherited) => quote! {
            #runtime::__private::v4::generic_field_metadata(
                &#descriptor_fields[#index],
                #variant_inherited,
                attributes,
                constraints,
                validators,
                serde,
            )
        },
        None => quote! {
            #runtime::__private::v4::field_metadata(
                &#descriptor_fields[#index],
                attributes,
                constraints,
                validators,
                serde,
            )
        },
    };
    quote! {
        {
            #sequence_assertion
            #map_assertion
            #constraint_assertions
            #identifier
            #unique
            #reference
            #key_part
            let validators: &'static [#runtime::ValidatorMetadata] =
                #runtime::__private::v4::leak_slice(::std::vec![#(#validators),*]);
            #codec
            #redact
            #serde
            #element_selector
            #map_key_selector
            #map_value_selector
            let constraints: &'static [#runtime::ConstraintMetadata] =
                #runtime::__private::v4::leak_slice(::std::vec![#(#constraints),*]);
            let mut attributes = ::std::vec::Vec::new();
            #(#occurrence_tokens)*
            #indexed
            let attributes: &'static [#runtime::FieldAttributeMetadata] =
                #runtime::__private::v4::leak_slice(attributes);
            fields.push(#metadata);
        }
    }
}

/// Generates runtime metadata for one normalized constraint.
fn expand_constraint(
    value: &ConstraintIr,
    has_element: bool,
    has_map_key: bool,
    has_map_value: bool,
    runtime: &TokenStream,
) -> TokenStream {
    match value {
        ConstraintIr::Text(value) => {
            let min_chars = option_number(value.min_chars);
            let max_chars = option_number(value.max_chars);
            let min_bytes = option_number(value.min_bytes);
            let max_bytes = option_number(value.max_bytes);
            let allowed = match value.allowed_chars.as_deref().unwrap_or("unicode") {
                "unicode" => quote!(#runtime::AllowedChars::Unicode),
                "printable_unicode" => {
                    quote!(#runtime::AllowedChars::PrintableUnicode)
                }
                "ascii" => quote!(#runtime::AllowedChars::Ascii),
                "printable_ascii" => {
                    quote!(#runtime::AllowedChars::PrintableAscii)
                }
                "code" => quote!(#runtime::AllowedChars::Code),
                _ => quote!(compile_error!("invalid allowed_chars value")),
            };
            let non_blank = value.non_blank;
            let format = value.format.as_deref().map_or_else(
                || quote!(None),
                |value| {
                    let value = match value {
                        "email" => quote!(#runtime::TextFormat::Email),
                        "cn_mobile" => quote!(#runtime::TextFormat::Mobile),
                        "uri" => quote!(#runtime::TextFormat::Uri),
                        "uuid" => quote!(#runtime::TextFormat::Uuid),
                        _ => quote!(compile_error!("invalid text format")),
                    };
                    quote!(Some(#value))
                },
            );
            quote!(#runtime::ConstraintMetadata::Text(#runtime::TextConstraint::new(
                #min_chars, #max_chars, #min_bytes, #max_bytes, #allowed, #non_blank, #format,
            )))
        }
        ConstraintIr::Decimal(value) => {
            let precision = option_number(value.precision);
            let scale = value.scale;
            let rounding = rounding_tokens(&value.rounding, runtime);
            let semantic = if value.money {
                quote!(#runtime::DecimalSemantic::Money)
            } else {
                quote!(#runtime::DecimalSemantic::Number)
            };
            let min = option_lit_str(&value.min);
            let max = option_lit_str(&value.max);
            let min_inclusive = value.min_inclusive;
            let max_inclusive = value.max_inclusive;
            quote!(#runtime::ConstraintMetadata::Decimal(
                #runtime::DecimalConstraint::new(#precision, #scale, #rounding, #semantic)
                    .with_bounds(#min, #max, #min_inclusive, #max_inclusive)
            ))
        }
        ConstraintIr::Time(value) => {
            let precision = match value.as_str() {
                "second" => quote!(#runtime::TemporalPrecision::Second),
                "millisecond" => {
                    quote!(#runtime::TemporalPrecision::Millisecond)
                }
                "microsecond" => {
                    quote!(#runtime::TemporalPrecision::Microsecond)
                }
                "nanosecond" => quote!(#runtime::TemporalPrecision::Nanosecond),
                _ => quote!(compile_error!("invalid time precision")),
            };
            quote!(#runtime::ConstraintMetadata::Time(#runtime::TimeConstraint::new(#precision)))
        }
        ConstraintIr::Sequence { min, max, unique } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let base = quote!(#runtime::SequenceConstraint::new(#min, #max, #unique));
            let value = if has_element {
                quote!(#base.with_element(element_selector))
            } else {
                base
            };
            quote!(#runtime::ConstraintMetadata::Sequence(#value))
        }
        ConstraintIr::Map { min, max } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let key = if has_map_key {
                quote!(Some(map_key_selector))
            } else {
                quote!(None)
            };
            let value = if has_map_value {
                quote!(Some(map_value_selector))
            } else {
                quote!(None)
            };
            quote!(#runtime::ConstraintMetadata::Map(#runtime::MapConstraint::new(#min, #max).with_selectors(#key, #value)))
        }
    }
}

/// Generates compile-time type-capability assertions for constraints.
fn expand_constraint_assertions(
    constraints: &[&ConstraintIr],
    target: TokenStream,
    runtime: &TokenStream,
) -> TokenStream {
    let assertions = constraints.iter().map(|constraint| match constraint {
        ConstraintIr::Text(_) => quote! {
            fn assert_text_target<T: #runtime::__private::v4::TextConstraintTarget + ?Sized>() {}
            assert_text_target::<#target>();
        },
        ConstraintIr::Decimal(_) => quote! {
            fn assert_decimal_target<T: #runtime::__private::v4::DecimalConstraintTarget>() {}
            assert_decimal_target::<#target>();
        },
        ConstraintIr::Time(_) => quote! {
            fn assert_temporal_target<T: #runtime::__private::v4::TemporalConstraintTarget>() {}
            assert_temporal_target::<#target>();
        },
        ConstraintIr::Sequence { min, max, unique } => {
            let length = (min.is_some() || max.is_some()).then(|| {
                quote! {
                    fn assert_variable_sequence<T: #runtime::__private::v4::VariableLengthSequenceTarget>() {}
                    assert_variable_sequence::<#target>();
                }
            });
            let uniqueness = unique.then(|| {
                quote! {
                    fn assert_unique_items_target<T: #runtime::__private::v4::UniqueItemsConstraintTarget>() {}
                    assert_unique_items_target::<#target>();
                }
            });
            quote! {
                fn assert_sequence_constraint<T: #runtime::__private::v4::SequenceConstraintTarget>() {}
                assert_sequence_constraint::<#target>();
                #length
                #uniqueness
            }
        }
        ConstraintIr::Map { .. } => quote! {
            fn assert_map_constraint<T: #runtime::__private::v4::MapConstraintTarget>() {}
            assert_map_constraint::<#target>();
        },
    });
    quote!(#(#assertions)*)
}

/// Generates runtime metadata for a collection selector.
fn expand_selector_metadata(
    value: &SelectorIr,
    value_type: &TokenStream,
    name: Ident,
    runtime: &TokenStream,
) -> TokenStream {
    let position = match value.position {
        SelectorPositionIr::Element => {
            quote!(#runtime::SelectorPosition::Element)
        }
        SelectorPositionIr::MapKey => {
            quote!(#runtime::SelectorPosition::MapKey)
        }
        SelectorPositionIr::MapValue => {
            quote!(#runtime::SelectorPosition::MapValue)
        }
    };
    let constraints = value
        .constraints
        .iter()
        .map(|constraint| expand_constraint(constraint, false, false, false, runtime));
    let constraint_refs: Vec<_> = value.constraints.iter().collect();
    let constraint_assertions = expand_constraint_assertions(&constraint_refs, value_type.clone(), runtime);
    let validators = value
        .validators
        .iter()
        .map(|validator| expand_validator(validator, runtime));
    let codec = value.codec.as_ref().map_or_else(
        || quote!(None),
        |codec| {
            let reference = codec_reference_expression(codec, value_type, runtime);
            quote!({
                let reference: &'static #runtime::CodecReference = #runtime::__private::v4::leak(#reference);
                Some(#runtime::__private::v4::leak(
                    #runtime::CodecMetadata::new(reference, #runtime::CodecSource::Selector(#position)),
                ) as &'static #runtime::CodecMetadata)
            })
        },
    );
    let redact = value.redact.as_ref().map_or_else(
        || quote!(None),
        |redact| {
            let expression = redact_expression(
                redact,
                match value.position {
                    SelectorPositionIr::Element => quote!(#runtime::RedactPosition::Element),
                    SelectorPositionIr::MapKey => quote!(#runtime::RedactPosition::MapKey),
                    SelectorPositionIr::MapValue => quote!(#runtime::RedactPosition::MapValue),
                },
                runtime,
            );
            quote!(Some(#runtime::__private::v4::leak(#expression) as &'static #runtime::RedactMetadata))
        },
    );
    quote! {
        #constraint_assertions
        let selector_constraints: &'static [#runtime::ConstraintMetadata] = #runtime::__private::v4::leak_slice(::std::vec![#(#constraints),*]);
        let selector_validators: &'static [#runtime::ValidatorMetadata] = #runtime::__private::v4::leak_slice(::std::vec![#(#validators),*]);
        let selector_codec = #codec;
        let selector_redact = #redact;
        let #name: &'static #runtime::SelectorMetadata = #runtime::__private::v4::leak(
            #runtime::SelectorMetadata::new(#position, selector_constraints, selector_validators, selector_codec, selector_redact),
        );
    }
}

/// Converts an optional tokenizable number into runtime option tokens.
fn option_number<T: ToTokens>(value: Option<T>) -> TokenStream {
    value.map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

/// Maps a validated rounding name to runtime enum tokens.
fn rounding_tokens(value: &str, runtime: &TokenStream) -> TokenStream {
    match value {
        "down" => quote!(#runtime::RoundingMode::Down),
        "up" => quote!(#runtime::RoundingMode::Up),
        "ceiling" => quote!(#runtime::RoundingMode::Ceiling),
        "floor" => quote!(#runtime::RoundingMode::Floor),
        "half_up" => quote!(#runtime::RoundingMode::HalfUp),
        "half_down" => quote!(#runtime::RoundingMode::HalfDown),
        "half_even" => quote!(#runtime::RoundingMode::HalfEven),
        "unnecessary" => quote!(#runtime::RoundingMode::Unnecessary),
        _ => quote!(compile_error!("invalid rounding mode")),
    }
}

/// Generates runtime metadata for one validator declaration.
fn expand_validator(validator: &ValidatorIr, runtime: &TokenStream) -> TokenStream {
    let id = &validator.id;
    let params = validator.params.iter().map(|(name, value)| {
        let value = expand_strategy_argument(value, runtime);
        quote!(#runtime::NamedValidationArgument::new(#name, #value))
    });
    let depends_on = validator.depends_on.iter().map(|path| expand_field_path(path, runtime));
    let dependency_bindings = validator.dependency_bindings.iter().map(|(name, path)| {
        let path = expand_field_path(path, runtime);
        quote!(#runtime::DependencyBindingMetadata::new(#name, #path))
    });
    let target = match validator.target {
        TargetModeIr::Value => quote!(#runtime::TargetMode::Value),
        TargetModeIr::Container => quote!(#runtime::TargetMode::Container),
    };
    let on_none = match validator.on_none {
        OnNoneIr::Skip => quote!(#runtime::OnNone::Skip),
        OnNoneIr::Reject => quote!(#runtime::OnNone::Reject),
    };
    let constructor = if validator.dependency_bindings.is_empty()
        && matches!(validator.target, TargetModeIr::Value)
        && matches!(validator.on_none, OnNoneIr::Skip)
    {
        quote!(#runtime::ValidatorMetadata::new(#id, params, depends_on))
    } else {
        quote!(#runtime::ValidatorMetadata::new_bound(
            #id,
            params,
            depends_on,
            dependency_bindings,
            #target,
            #on_none,
        ))
    };
    quote!({
        let params: &'static [#runtime::NamedValidationArgument<'static>] = #runtime::__private::v4::leak_slice(::std::vec![#(#params),*]);
        let depends_on: &'static [#runtime::PropertyPath<'static>] = #runtime::__private::v4::leak_slice(::std::vec![#(#depends_on),*]);
        let dependency_bindings: &'static [#runtime::DependencyBindingMetadata] = #runtime::__private::v4::leak_slice(::std::vec![#(#dependency_bindings),*]);
        #constructor
    })
}

/// Generates runtime tokens for one validator strategy argument.
fn expand_strategy_argument(value: &StrategyArgumentIr, runtime: &TokenStream) -> TokenStream {
    match value {
        StrategyArgumentIr::Bool(value) => {
            quote!(#runtime::ValidationArgument::Bool(#value))
        }
        StrategyArgumentIr::Integer(value) => {
            quote!(#runtime::ValidationArgument::Integer(#value))
        }
        StrategyArgumentIr::Unsigned(value) => {
            quote!(#runtime::ValidationArgument::Unsigned(#value))
        }
        StrategyArgumentIr::String(value) => {
            quote!(#runtime::ValidationArgument::String(#value))
        }
        StrategyArgumentIr::BoolList(values) => {
            quote!(#runtime::ValidationArgument::BoolList(&[#(#values),*]))
        }
        StrategyArgumentIr::IntegerList(values) => {
            quote!(#runtime::ValidationArgument::IntegerList(&[#(#values),*]))
        }
        StrategyArgumentIr::UnsignedList(values) => {
            quote!(#runtime::ValidationArgument::UnsignedList(&[#(#values),*]))
        }
        StrategyArgumentIr::StringList(values) => {
            quote!(#runtime::ValidationArgument::StringList(&[#(#values),*]))
        }
    }
}

/// Generates runtime metadata for one relationship declaration.
fn expand_reference(reference: &ReferenceIr, runtime: &TokenStream) -> TokenStream {
    let target = match &reference.target {
        ReferenceTargetIr::RustType(ty) => {
            quote!(#runtime::DeclaredEntityTarget::RustType(#runtime::TypeMetadata::of::<#ty>))
        }
        ReferenceTargetIr::ModelId(id) => {
            quote!(#runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id)))
        }
    };
    let selection = reference.property.as_ref().map_or_else(
        || quote!(#runtime::ReferenceSelection::Entity),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(#runtime::ReferenceSelection::Property(#path))
        },
    );
    let same_as = reference.same_as.as_ref().map_or_else(
        || quote!(None),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(Some(#runtime::__private::v4::leak(#path) as &'static #runtime::PropertyPath<'static>))
        },
    );
    let existing = reference.existing;
    quote! {
        let reference_target: &'static #runtime::DeclaredEntityTarget = #runtime::__private::v4::leak(#target);
        let reference_selection: &'static #runtime::ReferenceSelection = #runtime::__private::v4::leak(#selection);
        let reference: &'static #runtime::FieldReferenceMetadata = #runtime::__private::v4::leak(
            #runtime::FieldReferenceMetadata::new(reference_target, reference_selection, #existing, #same_as),
        );
    }
}

/// Generates runtime metadata for one redaction declaration.
fn expand_redact(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let expression = redact_expression(redact, position, runtime);
    quote! {
        let redact: &'static #runtime::RedactMetadata = #runtime::__private::v4::leak(#expression);
    }
}

/// Generates the redaction expression associated with a redaction mode.
fn redact_expression(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let (sensitivity, mode) = match &redact.mode {
        RedactModeIr::Level(level) => {
            let sensitivity = match level.as_str() {
                "low" => quote!(#runtime::Sensitivity::Low),
                "medium" => quote!(#runtime::Sensitivity::Medium),
                "high" => quote!(#runtime::Sensitivity::High),
                "secret" => quote!(#runtime::Sensitivity::Secret),
                _ => quote!(compile_error!("redact level must be low, medium, high, or secret")),
            };
            (quote!(Some(#sensitivity)), quote!(#runtime::RedactModeMetadata::Level))
        }
        RedactModeIr::Skip => (quote!(None), quote!(#runtime::RedactModeMetadata::Skip)),
        RedactModeIr::Nested => (quote!(None), quote!(#runtime::RedactModeMetadata::Nested)),
        RedactModeIr::Map => (quote!(None), quote!(#runtime::RedactModeMetadata::Map)),
        RedactModeIr::KeyedBy(field) => (quote!(None), quote!(#runtime::RedactModeMetadata::KeyedBy(#field))),
        RedactModeIr::Json => (quote!(None), quote!(#runtime::RedactModeMetadata::Json)),
    };
    quote!(#runtime::RedactMetadata::new(#sensitivity, #mode, #position))
}

/// Generates runtime metadata for a declared value codec.
fn codec_reference_expression<T: ToTokens>(codec: &CodecIr, value_type: &T, runtime: &TokenStream) -> TokenStream {
    match codec {
        CodecIr::DeclaredId(id) => {
            quote!(#runtime::CodecReference::DeclaredId(#id))
        }
        CodecIr::RustType(ty) => {
            quote!(#runtime::CodecReference::RustType(#runtime::__private::v4::leak(
                #runtime::__private::v4::ValueCodecDescriptor::of::<#ty, #value_type>(),
            )))
        }
    }
}

/// Generates runtime metadata for one Serde behavior declaration.
fn expand_serde(value: &SerdeIr, runtime: &TokenStream) -> TokenStream {
    let serialize_name = option_lit_str(&value.serialize_name);
    let deserialize_name = option_lit_str(&value.deserialize_name);
    let skip_serializing = value.skip_serializing;
    let skip_deserializing = value.skip_deserializing;
    let flatten = value.flatten;
    let with = option_lit_str(&value.with);
    let default = value.default;
    let default_source = if value.default_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.default {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    let omit_source = if value.omit_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.omit_suppressed {
        quote!(#runtime::SerdeBehaviorSource::Suppressed)
    } else if value.explicit_skip_serializing_if {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    quote! {
        let serde: &'static #runtime::SerdeFieldMetadata = #runtime::__private::v4::leak(
            #runtime::SerdeFieldMetadata::new(#serialize_name, #deserialize_name, #skip_serializing, #skip_deserializing, #flatten, #with, #default)
                .with_sources(#default_source, #omit_source),
        );
    }
}

/// Converts an owned field path into runtime path tokens.
fn expand_field_path(path: &[String], runtime: &TokenStream) -> TokenStream {
    quote!(#runtime::PropertyPath::new(&[#(#path),*]))
}

/// Converts an optional string literal into runtime option tokens.
fn option_lit_str(value: &Option<LitStr>) -> TokenStream {
    value
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::LitStr;
    use syn::Type;
    use syn::parse_quote;

    use super::codec_reference_expression;
    use super::expand_constraint;
    use super::expand_redact;
    use super::expand_reference;
    use super::expand_serde;
    use super::expand_strategy_argument;
    use super::expand_validator;
    use super::rounding_tokens;
    use crate::ir::declaration::CodecIr;
    use crate::ir::declaration::ConstraintIr;
    use crate::ir::declaration::DecimalConstraintIr;
    use crate::ir::declaration::RedactIr;
    use crate::ir::declaration::RedactModeIr;
    use crate::ir::declaration::ReferenceIr;
    use crate::ir::declaration::ReferenceTargetIr;
    use crate::ir::declaration::SerdeIr;
    use crate::ir::declaration::StrategyArgumentIr;
    use crate::ir::declaration::TextConstraintIr;
    use crate::ir::declaration::ValidatorIr;

    /// Exercises all code-generation branches for field metadata primitives.
    #[test]
    fn test_expand_field_metadata_branches() {
        let runtime: TokenStream = quote!(runtime);
        let literal = |value: &str| LitStr::new(value, Span::call_site());

        for allowed in [
            "unicode",
            "printable_unicode",
            "ascii",
            "printable_ascii",
            "code",
            "invalid",
        ] {
            for format in [
                None,
                Some("email"),
                Some("cn_mobile"),
                Some("uri"),
                Some("uuid"),
                Some("invalid"),
            ] {
                let value = ConstraintIr::Text(TextConstraintIr {
                    min_chars: Some(1),
                    max_chars: Some(2),
                    min_bytes: Some(1),
                    max_bytes: Some(4),
                    allowed_chars: Some(allowed.to_owned()),
                    non_blank: true,
                    format: format.map(str::to_owned),
                });
                assert!(!expand_constraint(&value, false, false, false, &runtime).is_empty());
            }
        }
        for rounding in [
            "down",
            "up",
            "ceiling",
            "floor",
            "half_up",
            "half_down",
            "half_even",
            "unnecessary",
            "invalid",
        ] {
            assert!(!rounding_tokens(rounding, &runtime).is_empty());
            let value = ConstraintIr::Decimal(DecimalConstraintIr {
                precision: Some(8),
                scale: 2,
                rounding: rounding.to_owned(),
                money: rounding == "unnecessary",
                min: Some(literal("1.0")),
                max: Some(literal("2.0")),
                min_inclusive: false,
                max_inclusive: true,
            });
            assert!(!expand_constraint(&value, false, false, false, &runtime).is_empty());
        }
        for precision in ["second", "millisecond", "microsecond", "nanosecond", "invalid"] {
            let value = ConstraintIr::Time(precision.to_owned());
            assert!(!expand_constraint(&value, false, false, false, &runtime).is_empty());
        }
        let sequence = ConstraintIr::Sequence {
            min: Some(1),
            max: Some(2),
            unique: true,
        };
        assert!(!expand_constraint(&sequence, true, false, false, &runtime).is_empty());
        let map = ConstraintIr::Map {
            min: Some(1),
            max: Some(2),
        };
        assert!(!expand_constraint(&map, false, true, true, &runtime).is_empty());

        let strategy_values = [
            StrategyArgumentIr::Bool(true),
            StrategyArgumentIr::Integer(-1),
            StrategyArgumentIr::Unsigned(1),
            StrategyArgumentIr::String(literal("value")),
            StrategyArgumentIr::BoolList(vec![true]),
            StrategyArgumentIr::IntegerList(vec![-1]),
            StrategyArgumentIr::UnsignedList(vec![1]),
            StrategyArgumentIr::StringList(vec![literal("value")]),
        ];
        for value in &strategy_values {
            assert!(!expand_strategy_argument(value, &runtime).is_empty());
        }
        let validator = ValidatorIr {
            id: literal("example.rule"),
            params: strategy_values
                .into_iter()
                .enumerate()
                .map(|(index, value)| (format!("value_{index}"), value))
                .collect(),
            depends_on: vec![vec!["owner".to_owned(), "id".to_owned()]],
            dependency_bindings: Vec::new(),
            target: Default::default(),
            on_none: Default::default(),
        };
        assert!(!expand_validator(&validator, &runtime).is_empty());

        let ty: Type = parse_quote!(Codec);
        let references = [
            ReferenceIr {
                target: ReferenceTargetIr::RustType(Box::new(parse_quote!(Owner))),
                property: None,
                existing: true,
                same_as: None,
            },
            ReferenceIr {
                target: ReferenceTargetIr::ModelId(literal("example.Owner")),
                property: Some(vec!["id".to_owned()]),
                existing: false,
                same_as: Some(vec!["owner".to_owned(), "id".to_owned()]),
            },
        ];
        for reference in &references {
            assert!(!expand_reference(reference, &runtime).is_empty());
        }
        assert!(!codec_reference_expression(&CodecIr::RustType(Box::new(ty)), &quote!(String), &runtime).is_empty());
        assert!(
            !codec_reference_expression(
                &CodecIr::DeclaredId(literal("example.codec")),
                &quote!(String),
                &runtime
            )
            .is_empty()
        );

        for mode in [
            RedactModeIr::Level("low".to_owned()),
            RedactModeIr::Level("medium".to_owned()),
            RedactModeIr::Level("high".to_owned()),
            RedactModeIr::Level("secret".to_owned()),
            RedactModeIr::Level("invalid".to_owned()),
            RedactModeIr::Skip,
            RedactModeIr::Nested,
            RedactModeIr::Map,
            RedactModeIr::KeyedBy("owner".to_owned()),
            RedactModeIr::Json,
        ] {
            assert!(!expand_redact(&RedactIr { mode }, quote!(position), &runtime).is_empty());
        }

        let serde_values = [
            SerdeIr::default(),
            SerdeIr {
                serialize_name: Some(literal("out")),
                deserialize_name: Some(literal("in")),
                skip_serializing: true,
                skip_deserializing: true,
                flatten: true,
                with: Some(literal("helper")),
                default: true,
                explicit_skip_serializing_if: false,
                default_from_model: true,
                omit_from_model: true,
                omit_suppressed: false,
            },
            SerdeIr {
                default: true,
                explicit_skip_serializing_if: true,
                omit_suppressed: true,
                ..SerdeIr::default()
            },
        ];
        for value in &serde_values {
            assert!(!expand_serde(value, &runtime).is_empty());
        }
    }
}
