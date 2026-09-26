// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Binding of metadata constraints to the shared standard rule set.

// qubit-style: allow multiple-public-types

use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;

use crate::metadata::AllowedChars;
use crate::metadata::ConstraintMetadata;
use crate::metadata::TemporalPrecision;
use crate::metadata::TextFormat;
use crate::validation::ConstraintRuleRef;

mod standard_rule;

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;
    use qubit_validation_rules::ids;
    use qubit_validation_rules::registrations;
    use qubit_validator::BindErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::ValidationArgument;
    use qubit_validator::ValidatorRegistry;

    use super::StandardRule;
    use super::build_builtin_registry;
    use super::visit_rules;
    use crate::metadata::AllowedChars;
    use crate::metadata::ConstraintMetadata;
    use crate::metadata::TemporalPrecision;
    use crate::metadata::TextConstraint;
    use crate::metadata::TextFormat;
    use crate::metadata::TimeConstraint;

    #[test]
    fn test_builtin_registry_rejects_duplicate_declarations() {
        let registration = registrations()[0];

        let error = build_builtin_registry(vec![registration, registration])
            .expect_err("duplicate built-in IDs must not form a registry");

        assert_eq!(error.kind(), BindErrorKind::InvalidDeclaration);
    }

    /// Checks every temporal token against the actual registered signature.
    #[test]
    fn test_all_temporal_precisions_map_to_bindable_rules() {
        let registry = ValidatorRegistry::from_registrations(registrations()).expect("built-in registry");
        for precision in [
            TemporalPrecision::Second,
            TemporalPrecision::Millisecond,
            TemporalPrecision::Microsecond,
            TemporalPrecision::Nanosecond,
        ] {
            let expected = match precision {
                TemporalPrecision::Second => "second",
                TemporalPrecision::Millisecond => "millisecond",
                TemporalPrecision::Microsecond => "microsecond",
                TemporalPrecision::Nanosecond => "nanosecond",
            };
            let constraint = ConstraintMetadata::Time(TimeConstraint::new(precision));
            let mut count = 0;
            visit_rules(&constraint, |rule| {
                count += 1;
                let StandardRule::Executable { id, args, .. } = rule else {
                    panic!("time precision must use the registry");
                };
                assert_eq!(id.as_str(), ids::TIME_PRECISION);
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].name(), "precision");
                assert_eq!(args[0].value(), ValidationArgument::String(expected));
                registry
                    .bind(id.as_str(), InputType::of::<NaiveTime>(), args)
                    .expect("time precision must bind");
            });
            assert_eq!(count, 1);
        }
    }

    /// Checks every character policy token against the text registration.
    #[test]
    fn test_all_character_policies_map_to_bindable_rules() {
        let registry = ValidatorRegistry::from_registrations(registrations()).expect("built-in registry");
        for set in [
            AllowedChars::Unicode,
            AllowedChars::PrintableUnicode,
            AllowedChars::Ascii,
            AllowedChars::PrintableAscii,
            AllowedChars::Code,
        ] {
            let expected = match set {
                AllowedChars::Unicode => None,
                AllowedChars::PrintableUnicode => Some("printable_unicode"),
                AllowedChars::Ascii => Some("ascii"),
                AllowedChars::PrintableAscii => Some("printable_ascii"),
                AllowedChars::Code => Some("code"),
            };
            let constraint = ConstraintMetadata::Text(TextConstraint::new(None, None, None, None, set, false, None));
            let mut count = 0;
            visit_rules(&constraint, |rule| {
                count += 1;
                let StandardRule::Executable { id, args, .. } = rule else {
                    panic!("character policy must use the registry");
                };
                assert_eq!(id.as_str(), ids::TEXT_ALLOWED_CHARS);
                assert_eq!(args.len(), 1);
                assert_eq!(args[0].name(), "set");
                assert_eq!(
                    args[0].value(),
                    ValidationArgument::String(expected.expect("non-default policy"))
                );
                registry
                    .bind(id.as_str(), InputType::Text, args)
                    .expect("character policy must bind");
            });
            assert_eq!(count, usize::from(expected.is_some()));
        }
    }

    /// Checks every format's identity and parameterless text signature.
    #[test]
    fn test_all_text_formats_map_to_bindable_rules() {
        let registry = ValidatorRegistry::from_registrations(registrations()).expect("built-in registry");
        for format in [
            TextFormat::EmailAscii,
            TextFormat::Mobile,
            TextFormat::Uri,
            TextFormat::Uuid,
        ] {
            let expected = match format {
                TextFormat::EmailAscii => ids::TEXT_EMAIL_ASCII,
                TextFormat::Mobile => ids::TEXT_CHINA_MOBILE_STRUCTURE,
                TextFormat::Uri => ids::TEXT_URI,
                TextFormat::Uuid => ids::TEXT_UUID,
            };
            let constraint = ConstraintMetadata::Text(TextConstraint::new(
                None,
                None,
                None,
                None,
                AllowedChars::Unicode,
                false,
                Some(format),
            ));
            let mut count = 0;
            visit_rules(&constraint, |rule| {
                count += 1;
                let StandardRule::Executable { id, args, .. } = rule else {
                    panic!("text format must use the registry");
                };
                assert_eq!(id.as_str(), expected);
                assert!(args.is_empty());
                registry
                    .bind(id.as_str(), InputType::Text, args)
                    .expect("text format must bind");
            });
            assert_eq!(count, 1);
        }
    }
}

use standard_rule::StandardRule;

/// Stable identity of metadata's typed sequence equality adapter.
pub(crate) const SEQUENCE_UNIQUE_ID: ValidatorId = ValidatorId::new("qubit.rules.collection.unique");

/// Whether a standard binding validates the field value or its item count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StandardTarget {
    /// Pass the field value to the rule.
    Value,
    /// Pass the borrowed sequence length as a `usize`.
    SequenceCount,
    /// Pass the borrowed map length as a `usize`.
    MapCount,
}

/// One executable standard constraint occurrence.
pub(crate) struct StandardBinding {
    /// The prepared standard rule.
    pub(crate) validator: BoundValidator,
    /// The input projection required by this rule.
    pub(crate) target: StandardTarget,
}

/// Combines the built-in rules with the caller's local rules.
///
/// Built-in IDs cannot be overridden. Returns a canonical registry and one
/// diagnostic per conflicting caller registration, allowing independent
/// declarations to be checked before the plan is rejected.
///
/// # Errors
/// Returns `InvalidDeclaration` if the built-in definitions themselves contain
/// duplicate IDs and cannot form a registry.
pub(crate) fn registry(validators: &ValidatorRegistry) -> Result<(ValidatorRegistry, Vec<BindError>), BindError> {
    registry_with_builtins(registrations(), validators)
}

fn registry_with_builtins(
    mut registrations: Vec<ValidatorRegistration>,
    validators: &ValidatorRegistry,
) -> Result<(ValidatorRegistry, Vec<BindError>), BindError> {
    let builtin_count = registrations.len();
    let mut errors = Vec::new();
    for registration in validators.registrations() {
        if registrations[..builtin_count]
            .iter()
            .any(|builtin| builtin.id() == registration.id())
        {
            errors.push(BindError::new(BindErrorKind::InvalidDeclaration).with_rule(registration.id()));
        } else {
            registrations.push(*registration);
        }
    }
    build_builtin_registry(registrations).map(|registry| (registry, errors))
}

/// Builds the canonical registry and maps malformed built-in declarations to
/// the metadata binding error category.
fn build_builtin_registry(registrations: Vec<ValidatorRegistration>) -> Result<ValidatorRegistry, BindError> {
    ValidatorRegistry::from_registrations(registrations).map_err(|_| BindError::new(BindErrorKind::InvalidDeclaration))
}

/// Binds the executable portion of one metadata constraint.
///
/// Reflection-specific sequence equality is bound separately by the metadata
/// executor; this registry binding only returns registry-backed rules.
pub(crate) fn bind(
    constraint: &ConstraintMetadata,
    validators: &ValidatorRegistry,
    input: InputType,
) -> Result<Vec<StandardBinding>, Vec<BindError>> {
    let mut bindings = Vec::new();
    let mut errors = Vec::new();
    visit_rules(constraint, |rule| match rule {
        StandardRule::Executable { id, target, args } => {
            match validators.bind(id.as_str(), input_type(target, input), args) {
                Ok(validator) => bindings.push(StandardBinding { validator, target }),
                Err(error) => errors.push(error),
            }
        }
        StandardRule::SequenceUnique { .. } => {}
    });
    if errors.is_empty() { Ok(bindings) } else { Err(errors) }
}

/// Returns every known rule mapping in execution order without consulting a
/// registry or invoking a validator. Unknown backend mappings contribute no
/// reference.
pub(crate) fn rule_refs(constraint: &ConstraintMetadata) -> Vec<ConstraintRuleRef> {
    let mut refs = Vec::new();
    visit_rules(constraint, |rule| refs.push(rule.diagnostic_ref()));
    refs
}

/// Visits canonical rule mappings synchronously. Argument slices are valid
/// only during the callback; no getter, registry or validator is invoked here.
fn visit_rules(constraint: &ConstraintMetadata, mut visitor: impl FnMut(StandardRule<'_>)) {
    match constraint {
        ConstraintMetadata::Text(text) => {
            if text.is_non_blank() {
                visit_rule(&mut visitor, ids::TEXT_NON_BLANK, &[], StandardTarget::Value);
            }
            if text.min_chars().is_some() || text.max_chars().is_some() {
                let args = optional_u32_args(text.min_chars(), text.max_chars());
                visit_rule(&mut visitor, ids::TEXT_CHAR_LENGTH, &args, StandardTarget::Value);
            }
            if text.min_bytes().is_some() || text.max_bytes().is_some() {
                let args = optional_u32_args(text.min_bytes(), text.max_bytes());
                visit_rule(&mut visitor, ids::TEXT_BYTE_LENGTH, &args, StandardTarget::Value);
            }
            if !matches!(text.allowed_chars(), AllowedChars::Unicode) {
                let args = [NamedValidationArgument::new(
                    "set",
                    ValidationArgument::String(allowed_chars(text.allowed_chars())),
                )];
                visit_rule(&mut visitor, ids::TEXT_ALLOWED_CHARS, &args, StandardTarget::Value);
            }
            if let Some(format) = text.format() {
                let id = match format {
                    TextFormat::EmailAscii => ids::TEXT_EMAIL_ASCII,
                    TextFormat::Mobile => ids::TEXT_CHINA_MOBILE_STRUCTURE,
                    TextFormat::Uri => ids::TEXT_URI,
                    TextFormat::Uuid => ids::TEXT_UUID,
                };
                visit_rule(&mut visitor, id, &[], StandardTarget::Value);
            }
        }
        ConstraintMetadata::Sequence(sequence) => {
            if sequence.min_items().is_some() || sequence.max_items().is_some() {
                let args = optional_usize_args(sequence.min_items(), sequence.max_items());
                visit_rule(
                    &mut visitor,
                    ids::COLLECTION_ITEM_COUNT,
                    &args,
                    StandardTarget::SequenceCount,
                );
            }
            if sequence.unique_items() {
                visitor(StandardRule::SequenceUnique { id: SEQUENCE_UNIQUE_ID });
            }
        }
        ConstraintMetadata::Map(map) => {
            if map.min_entries().is_some() || map.max_entries().is_some() {
                let args = optional_usize_args(map.min_entries(), map.max_entries());
                visit_rule(
                    &mut visitor,
                    ids::COLLECTION_ITEM_COUNT,
                    &args,
                    StandardTarget::MapCount,
                );
            }
        }
        ConstraintMetadata::Decimal(decimal) => {
            let mut args = Vec::with_capacity(6);
            if let Some(precision) = decimal.precision() {
                args.push(NamedValidationArgument::new(
                    "precision",
                    ValidationArgument::Unsigned(u128::from(precision)),
                ));
            }
            args.push(NamedValidationArgument::new(
                "scale",
                ValidationArgument::Unsigned(u128::from(decimal.scale())),
            ));
            if let Some(min) = decimal.min() {
                args.push(NamedValidationArgument::new("min", ValidationArgument::String(min)));
            }
            if let Some(max) = decimal.max() {
                args.push(NamedValidationArgument::new("max", ValidationArgument::String(max)));
            }
            args.push(NamedValidationArgument::new(
                "min_inclusive",
                ValidationArgument::Bool(decimal.min_inclusive()),
            ));
            args.push(NamedValidationArgument::new(
                "max_inclusive",
                ValidationArgument::Bool(decimal.max_inclusive()),
            ));
            visit_rule(&mut visitor, ids::DECIMAL_VALUE, &args, StandardTarget::Value);
        }
        ConstraintMetadata::Time(time) => {
            let precision = match time.precision() {
                TemporalPrecision::Second => "second",
                TemporalPrecision::Millisecond => "millisecond",
                TemporalPrecision::Microsecond => "microsecond",
                TemporalPrecision::Nanosecond => "nanosecond",
            };
            let args = [NamedValidationArgument::new(
                "precision",
                ValidationArgument::String(precision),
            )];
            visit_rule(&mut visitor, ids::TIME_PRECISION, &args, StandardTarget::Value);
        }
    }
}

/// Emits one executable mapping while the caller's argument slice is alive.
fn visit_rule(
    visitor: &mut impl FnMut(StandardRule<'_>),
    id: &'static str,
    args: &[NamedValidationArgument<'static>],
    target: StandardTarget,
) {
    visitor(StandardRule::Executable {
        id: ValidatorId::new(id),
        target,
        args,
    });
}

/// Returns the runtime input type required by a standard target.
fn input_type(target: StandardTarget, value: InputType) -> InputType {
    match target {
        StandardTarget::Value => value,
        StandardTarget::SequenceCount | StandardTarget::MapCount => InputType::of::<usize>(),
    }
}

/// Builds optional unsigned-32-bit bound arguments in declaration order.
fn optional_u32_args(min: Option<u32>, max: Option<u32>) -> Vec<NamedValidationArgument<'static>> {
    let mut args = Vec::with_capacity(2);
    if let Some(value) = min {
        args.push(NamedValidationArgument::new(
            "min",
            ValidationArgument::Unsigned(u128::from(value)),
        ));
    }
    if let Some(value) = max {
        args.push(NamedValidationArgument::new(
            "max",
            ValidationArgument::Unsigned(u128::from(value)),
        ));
    }
    args
}

/// Builds optional machine-word bound arguments in declaration order.
fn optional_usize_args(min: Option<usize>, max: Option<usize>) -> Vec<NamedValidationArgument<'static>> {
    let mut args = Vec::with_capacity(2);
    if let Some(value) = min {
        args.push(NamedValidationArgument::new(
            "min",
            ValidationArgument::Unsigned(value as u128),
        ));
    }
    if let Some(value) = max {
        args.push(NamedValidationArgument::new(
            "max",
            ValidationArgument::Unsigned(value as u128),
        ));
    }
    args
}

/// Maps a character policy to the runtime rule vocabulary.
const fn allowed_chars(value: AllowedChars) -> &'static str {
    match value {
        AllowedChars::Unicode => "unicode",
        AllowedChars::PrintableUnicode => "printable_unicode",
        AllowedChars::Ascii => "ascii",
        AllowedChars::PrintableAscii => "printable_ascii",
        AllowedChars::Code => "code",
    }
}
