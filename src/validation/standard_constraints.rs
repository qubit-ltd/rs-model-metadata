//! Binding of metadata constraints to the shared standard rule set.

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidatorRegistry;

use crate::AllowedChars;
use crate::ConstraintMetadata;
use crate::TextFormat;

/// Whether a standard binding validates the field value or its item count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StandardTarget {
    /// Pass the field value to the rule.
    Value,
    /// Pass the borrowed sequence length as a `usize`.
    SequenceCount,
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
/// Built-in IDs are deliberately not overrideable: the registry's duplicate
/// check makes an accidental semantic change visible during plan binding.
pub(crate) fn registry(validators: &ValidatorRegistry) -> Result<ValidatorRegistry, BindError> {
    let mut registrations = qubit_validation_rules::registrations();
    registrations.extend(validators.registrations().iter().copied());
    ValidatorRegistry::from_registrations(registrations)
        .map_err(|_| BindError::new(BindErrorKind::InvalidDeclaration))
}

/// Binds the executable portion of one metadata constraint.
///
/// Constraints which require reflection-specific behavior, such as duplicate
/// detection for arbitrary erased values, fail explicitly at bind time. They
/// are never silently accepted and skipped.
pub(crate) fn bind(
    constraint: &ConstraintMetadata,
    validators: &ValidatorRegistry,
) -> Result<Vec<StandardBinding>, Vec<BindError>> {
    let mut bindings = Vec::new();
    let mut errors = Vec::new();
    match constraint {
        ConstraintMetadata::Text(text) => {
            if text.is_non_blank() {
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    "qubit.rules.text.non_blank",
                    &[],
                    StandardTarget::Value,
                );
            }
            if text.min_chars().is_some() || text.max_chars().is_some() {
                let args = optional_u32_args(text.min_chars(), text.max_chars());
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    "qubit.rules.text.char_length",
                    &args,
                    StandardTarget::Value,
                );
            }
            if text.min_bytes().is_some() || text.max_bytes().is_some() {
                let args = optional_u32_args(text.min_bytes(), text.max_bytes());
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    "qubit.rules.text.byte_length",
                    &args,
                    StandardTarget::Value,
                );
            }
            if !matches!(text.allowed_chars(), AllowedChars::Unicode) {
                let args = [NamedValidationArgument::new(
                    "set",
                    ValidationArgument::String(allowed_chars(text.allowed_chars())),
                )];
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    "qubit.rules.text.allowed_chars",
                    &args,
                    StandardTarget::Value,
                );
            }
            if let Some(format) = text.format() {
                let id = match format {
                    TextFormat::Email => "qubit.rules.text.email_ascii",
                    TextFormat::Mobile => "qubit.rules.text.china_mobile_structure",
                    TextFormat::Uri => "qubit.rules.text.uri",
                    TextFormat::Uuid => "qubit.rules.text.uuid",
                };
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    id,
                    &[],
                    StandardTarget::Value,
                );
            }
        }
        ConstraintMetadata::Sequence(sequence) => {
            if sequence.min_items().is_some() || sequence.max_items().is_some() {
                let args = optional_usize_args(sequence.min_items(), sequence.max_items());
                bind_one(
                    &mut bindings,
                    &mut errors,
                    validators,
                    "qubit.rules.collection.item_count",
                    &args,
                    StandardTarget::SequenceCount,
                );
            }
            if sequence.unique_items() {
                errors.push(
                    BindError::new(BindErrorKind::UnsupportedConstraint).with_rule(
                        qubit_validator::ValidatorId::new("qubit.rules.collection.unique"),
                    ),
                );
            }
        }
        ConstraintMetadata::Map(_) => {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
        }
        ConstraintMetadata::Decimal(_) | ConstraintMetadata::Time(_) => {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
        }
    }
    if errors.is_empty() {
        Ok(bindings)
    } else {
        Err(errors)
    }
}

fn bind_one(
    bindings: &mut Vec<StandardBinding>,
    errors: &mut Vec<BindError>,
    validators: &ValidatorRegistry,
    id: &'static str,
    args: &[NamedValidationArgument<'_>],
    target: StandardTarget,
) {
    match validators.bind(id, input_type(target), args) {
        Ok(validator) => bindings.push(StandardBinding { validator, target }),
        Err(error) => errors.push(error),
    }
}

fn input_type(target: StandardTarget) -> qubit_validator::InputType {
    match target {
        StandardTarget::Value => qubit_validator::InputType::Text,
        StandardTarget::SequenceCount => qubit_validator::InputType::of::<usize>(),
    }
}

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

fn optional_usize_args(
    min: Option<usize>,
    max: Option<usize>,
) -> Vec<NamedValidationArgument<'static>> {
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

const fn allowed_chars(value: AllowedChars) -> &'static str {
    match value {
        AllowedChars::Unicode => "unicode",
        AllowedChars::PrintableUnicode => "printable_unicode",
        AllowedChars::Ascii => "ascii",
        AllowedChars::PrintableAscii => "printable_ascii",
        AllowedChars::Code => "code",
    }
}
