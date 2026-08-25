// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Local semantic validation for normalized derive input.

use std::slice::from_ref;

use proc_macro2::Span;
use syn::Error;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Type;
use syn::TypePath;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use crate::attribute::FieldName;
use crate::attribute::SequenceAttribute;
use crate::attribute::SpannedValue;
use crate::attribute::TextAttribute;
use crate::normalize::DecimalIr;
use crate::normalize::DecimalSemantic;
use crate::normalize::ElementConstraintIr;
use crate::normalize::ElementIr;
use crate::normalize::FieldAttributeIr;
use crate::normalize::FieldIr;
use crate::normalize::ModelAttributeIr;
use crate::normalize::ModelIr;
use crate::normalize::ModelShapeIr;
use crate::normalize::NamedFieldsIr;
use crate::normalize::PrimaryKeyIr;
use crate::normalize::UniqueIr;

/// Validates all independently checkable semantics of one normalized model.
///
/// # Parameters
///
/// - `model`: The normalized model whose local attributes and field references
///   are checked.
///
/// # Errors
///
/// Returns a combined [`syn::Error`] containing every independently located
/// invalid attribute, range, capability, conflict, or local field reference.
pub(crate) fn validate(model: &ModelIr) -> Result<()> {
    let mut errors = None;
    validate_model_id(model, &mut errors);
    validate_model_attribute_scope(model, &mut errors);
    validate_model_attributes(model, &mut errors);
    let fields = model_fields(model);
    for field in fields {
        validate_field(field, fields, &mut errors);
    }
    finish(errors)
}

/// Validates the required stable ID declared directly on the model type.
fn validate_model_id(model: &ModelIr, errors: &mut Option<Error>) {
    let Some(id) = model.id.first() else {
        push_error(
            errors,
            Error::new(
                model.ident.span(),
                "missing required model ID; add `#[Model(id = \"module.Type\")]`",
            ),
        );
        return;
    };
    for duplicate in model.id.iter().skip(1) {
        push_error(errors, Error::new(duplicate.span(), "duplicate model ID"));
    }
    if let Err(message) = validate_model_id_format(id.value().as_str()) {
        push_error(errors, Error::new(id.span(), message));
    } else {
        validate_model_id_type_name(id, &model.ident, errors);
    }
}

/// Validates the byte-level stable-ID grammar shared with the runtime crate.
fn validate_model_id_format(value: &str) -> core::result::Result<(), &'static str> {
    if value.is_empty() {
        return Err("model ID cannot be empty");
    }
    if value.split('.').any(str::is_empty) {
        return Err("model ID cannot contain empty segments");
    }
    let mut segments = value.split('.').peekable();
    while let Some(segment) = segments.next() {
        if segments.peek().is_some() {
            validate_model_id_module_segment(segment)?;
        } else {
            validate_model_id_type_segment(segment)?;
        }
    }
    Ok(())
}

/// Validates one ASCII snake-case module segment.
fn validate_model_id_module_segment(segment: &str) -> core::result::Result<(), &'static str> {
    if is_rust_keyword(segment) {
        return Err("model ID module segments cannot be Rust keywords");
    }
    let bytes = segment.as_bytes();
    if !matches!(bytes.first(), Some(b'a'..=b'z')) || matches!(bytes.last(), Some(b'_')) {
        return Err("model ID module segments must use ASCII snake_case");
    }
    let mut previous_underscore = false;
    for &byte in &bytes[1..] {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || (byte == b'_' && previous_underscore)
        {
            return Err("model ID module segments must use ASCII snake_case");
        }
        previous_underscore = byte == b'_';
    }
    Ok(())
}

/// Validates one ASCII UpperCamelCase final type segment.
fn validate_model_id_type_segment(segment: &str) -> core::result::Result<(), &'static str> {
    let bytes = segment.as_bytes();
    if !matches!(bytes.first(), Some(b'A'..=b'Z')) || !bytes[1..].iter().all(u8::is_ascii_alphanumeric) {
        return Err("model ID type segment must use ASCII UpperCamelCase");
    }
    Ok(())
}

/// Validates that the ID's final segment matches the derived Rust type name.
fn validate_model_id_type_name(id: &LitStr, ident: &Ident, errors: &mut Option<Error>) {
    let id_value = id.value();
    let actual_type_name = id_value.rsplit('.').next().unwrap_or_default();
    let expected_type_name = ident.unraw().to_string();
    if actual_type_name != expected_type_name {
        push_error(
            errors,
            Error::new(
                id.span(),
                format!("model ID type segment `{actual_type_name}` must match derived type `{expected_type_name}`"),
            ),
        );
    }
}

/// Returns whether a module segment is reserved by Rust 2024.
fn is_rust_keyword(segment: &str) -> bool {
    matches!(
        segment,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Returns the fields addressable by local model-level constraints.
#[must_use]
#[inline(always)]
fn model_fields(model: &ModelIr) -> &[FieldIr] {
    match &model.shape {
        ModelShapeIr::NamedStruct(fields) => fields,
        ModelShapeIr::Newtype(field) => from_ref(field.as_ref()),
        ModelShapeIr::UnitStruct | ModelShapeIr::FieldlessEnum(_) => &[],
    }
}

/// Rejects attributes written directly on declaration shapes other than named
/// structs.
fn validate_model_attribute_scope(model: &ModelIr, errors: &mut Option<Error>) {
    if matches!(model.shape, ModelShapeIr::NamedStruct(_)) {
        return;
    }
    for attribute in &model.attributes {
        push_error(
            errors,
            Error::new(
                model_attribute_span(attribute),
                format!(
                    "model-level `{}` constraints are only supported on named structs",
                    model_attribute_name(attribute)
                ),
            ),
        );
    }
}

/// Validates model constraints, their duplicate declarations, and their field
/// sets.
fn validate_model_attributes(model: &ModelIr, errors: &mut Option<Error>) {
    validate_primary_key_declarations(model, errors);
    validate_ownership_declarations(model, errors);
    validate_repeated_constraints(model, errors);
    validate_named_constraint_identifiers(model, errors);

    for attribute in &model.attributes {
        match attribute {
            ModelAttributeIr::PrimaryKey(value) => {
                validate_primary_key(value, model_fields(model), errors);
            }
            ModelAttributeIr::Unique(value) => {
                validate_unique(value, model_fields(model), errors);
            }
            ModelAttributeIr::Index(value) => {
                validate_named_fields("index", value, model_fields(model), errors);
            }
            ModelAttributeIr::Key(value) => {
                validate_named_fields("key", value, model_fields(model), errors);
            }
            ModelAttributeIr::Ownership(value) => {
                validate_duplicate_type_paths("owner", &value.owner, errors);
            }
        }
    }
    validate_unique_index_conflicts(model, errors);
    validate_reference_index_conflicts(model, errors);
}

/// Rejects an explicit index on a field that is already unique.
fn validate_unique_index_conflicts(model: &ModelIr, errors: &mut Option<Error>) {
    let unique_fields = model
        .attributes
        .iter()
        .filter_map(|attribute| match attribute {
            ModelAttributeIr::Unique(value) => value.fields.last().map(|field| field.name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for attribute in &model.attributes {
        let ModelAttributeIr::Index(index) = attribute else {
            continue;
        };
        if index.implicit {
            continue;
        }
        for (name, span) in &index.fields {
            if unique_fields.contains(&name.as_str()) {
                push_error(
                    errors,
                    Error::new(
                        *span,
                        format!("field `{name}` cannot declare both `unique` and `index`"),
                    ),
                );
            }
        }
    }
}

/// Rejects an explicit index on a field that declares a reference.
fn validate_reference_index_conflicts(model: &ModelIr, errors: &mut Option<Error>) {
    let reference_fields = model_fields(model)
        .iter()
        .filter(|field| {
            field
                .attributes
                .iter()
                .any(|attribute| matches!(attribute, FieldAttributeIr::Reference(_)))
        })
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();
    for attribute in &model.attributes {
        let ModelAttributeIr::Index(index) = attribute else {
            continue;
        };
        if index.implicit {
            continue;
        }
        for (name, span) in &index.fields {
            if reference_fields.contains(&name.as_str()) {
                push_error(
                    errors,
                    Error::new(
                        *span,
                        format!("field `{name}` cannot declare both `reference` and `index`"),
                    ),
                );
            }
        }
    }
}

/// Validates that explicitly named constraints are non-blank and unique within
/// their constraint category.
fn validate_named_constraint_identifiers(model: &ModelIr, errors: &mut Option<Error>) {
    for (index, attribute) in model.attributes.iter().enumerate() {
        let Some((kind, name)) = named_constraint_identifier(attribute) else {
            continue;
        };
        if name.value().trim().is_empty() {
            push_error(
                errors,
                Error::new(name.span(), format!("`{kind}` constraint name cannot be empty")),
            );
        }
        if model.attributes[..index].iter().any(|previous| {
            named_constraint_identifier(previous).is_some_and(|(previous_kind, previous_name)| {
                previous_kind == kind && previous_name.value() == name.value()
            })
        }) {
            push_error(
                errors,
                Error::new(
                    name.span(),
                    format!("duplicate `{kind}` constraint name `{}`", name.value()),
                ),
            );
        }
    }
}

/// Returns an explicitly declared model-constraint name and its category.
fn named_constraint_identifier(attribute: &ModelAttributeIr) -> Option<(&'static str, &LitStr)> {
    match attribute {
        ModelAttributeIr::Unique(value) => value.name.first().map(|name| ("unique", name)),
        ModelAttributeIr::Index(value) => value.name.first().map(|name| ("index", name)),
        ModelAttributeIr::Key(value) => value.name.first().map(|name| ("key", name)),
        ModelAttributeIr::PrimaryKey(_) | ModelAttributeIr::Ownership(_) => None,
    }
}

/// Rejects duplicate model primary keys and model/shorthand primary-key
/// conflicts.
fn validate_primary_key_declarations(model: &ModelIr, errors: &mut Option<Error>) {
    let primary_keys = model
        .attributes
        .iter()
        .enumerate()
        .filter_map(|(index, attribute)| match attribute {
            ModelAttributeIr::PrimaryKey(value) => Some((index, value)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let model_primary_keys = primary_keys
        .iter()
        .copied()
        .filter(|(index, _)| *index < model.model_attribute_count)
        .collect::<Vec<_>>();
    for (_, value) in model_primary_keys.iter().skip(1) {
        push_error(errors, Error::new(value.span, "duplicate `primary_key` attribute"));
    }
    if !model_primary_keys.is_empty() {
        for (_, value) in primary_keys
            .iter()
            .copied()
            .filter(|(index, _)| *index >= model.model_attribute_count)
        {
            push_error(
                errors,
                Error::new(
                    value.span,
                    "field `identifier` shorthand conflicts with the model-level `primary_key`",
                ),
            );
        }
    }
}

/// Rejects more than one ownership declaration for a model.
fn validate_ownership_declarations(model: &ModelIr, errors: &mut Option<Error>) {
    let mut seen = false;
    for attribute in &model.attributes {
        if let ModelAttributeIr::Ownership(value) = attribute {
            if seen {
                push_error(errors, Error::new(value.span, "duplicate `ownership` attribute"));
            }
            seen = true;
        }
    }
}

/// Rejects semantically identical unique, index, and key constraints.
fn validate_repeated_constraints(model: &ModelIr, errors: &mut Option<Error>) {
    for (index, attribute) in model.attributes.iter().enumerate() {
        let duplicate = model.attributes[..index]
            .iter()
            .any(|previous| same_repeatable_constraint(previous, attribute));
        if duplicate {
            push_error(
                errors,
                Error::new(
                    model_attribute_span(attribute),
                    format!("duplicate `{}` constraint", model_attribute_name(attribute)),
                ),
            );
        }
    }
}

/// Returns whether two repeatable model attributes describe the same
/// constraint.
fn same_repeatable_constraint(left: &ModelAttributeIr, right: &ModelAttributeIr) -> bool {
    match (left, right) {
        (ModelAttributeIr::Unique(left), ModelAttributeIr::Unique(right)) => {
            optional_names_equal(&left.name, &right.name)
                && left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(&right.fields)
                    .all(|(left, right)| left.name == right.name)
        }
        (ModelAttributeIr::Index(left), ModelAttributeIr::Index(right))
        | (ModelAttributeIr::Key(left), ModelAttributeIr::Key(right)) => {
            optional_names_equal(&left.name, &right.name)
                && left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(&right.fields)
                    .all(|((left, _), (right, _))| left == right)
        }
        _ => false,
    }
}

/// Compares the first logical-name occurrence of two normalized constraints.
fn optional_names_equal(left: &[LitStr], right: &[LitStr]) -> bool {
    match (left.first(), right.first()) {
        (Some(left), Some(right)) => left.value() == right.value(),
        (None, None) => true,
        _ => false,
    }
}

/// Validates a primary key's required, unique, existing, and generated fields.
fn validate_primary_key(value: &PrimaryKeyIr, fields: &[FieldIr], errors: &mut Option<Error>) {
    if value.fields.is_empty() {
        push_error(
            errors,
            Error::new(value.span, "`primary_key` requires at least one field"),
        );
    }
    for (index, field) in value.fields.iter().enumerate() {
        if value.fields[..index].iter().any(|previous| previous.name == field.name) {
            push_error(
                errors,
                Error::new(field.span, format!("duplicate primary-key field `{}`", field.name)),
            );
        } else if !field_exists(fields, &field.name) {
            push_error(
                errors,
                Error::new(field.span, format!("unknown model field `{}`", field.name)),
            );
        }
    }
    for (index, field) in value.generated.iter().enumerate() {
        if value.generated[..index]
            .iter()
            .any(|previous| previous.name == field.name)
        {
            push_error(
                errors,
                Error::new(field.span, format!("duplicate generated field `{}`", field.name)),
            );
        } else if !field_exists(fields, &field.name) {
            push_unknown_field(errors, field);
        } else if !value.fields.iter().any(|key| key.name == field.name) {
            push_error(
                errors,
                Error::new(
                    field.span,
                    format!("generated field `{}` is not part of this primary key", field.name),
                ),
            );
        }
    }
}

/// Validates a unique constraint's name, field set, and ignore-case references.
fn validate_unique(value: &UniqueIr, fields: &[FieldIr], errors: &mut Option<Error>) {
    validate_duplicate_literals("name", &value.name, errors);
    if value.fields.is_empty() {
        push_error(errors, Error::new(value.span, "`unique` requires at least one field"));
    }
    for (index, field) in value.fields.iter().enumerate() {
        if value.fields[..index].iter().any(|previous| previous.name == field.name) {
            push_error(
                errors,
                Error::new(field.span, format!("duplicate unique field `{}`", field.name)),
            );
        } else if !field_exists(fields, &field.name) {
            push_error(
                errors,
                Error::new(field.span, format!("unknown model field `{}`", field.name)),
            );
        }
    }
    for (index, field) in value.ignore_case.iter().enumerate() {
        if value.ignore_case[..index]
            .iter()
            .any(|previous| previous.name == field.name)
        {
            push_error(
                errors,
                Error::new(field.span, format!("duplicate ignore-case field `{}`", field.name)),
            );
            continue;
        }
        if !field_exists(fields, &field.name) {
            push_unknown_field(errors, field);
            continue;
        }
        if !value.fields.iter().any(|unique_field| unique_field.name == field.name) {
            push_error(
                errors,
                Error::new(
                    field.span,
                    format!(
                        "ignore-case field `{}` is not part of this unique constraint",
                        field.name
                    ),
                ),
            );
        }
    }
}

/// Validates an index or logical key's name and ordered field set.
fn validate_named_fields(kind: &str, value: &NamedFieldsIr, fields: &[FieldIr], errors: &mut Option<Error>) {
    validate_duplicate_literals("name", &value.name, errors);
    if value.fields.is_empty() {
        push_error(
            errors,
            Error::new(value.span, format!("`{kind}` requires at least one field")),
        );
    }
    for (index, (name, span)) in value.fields.iter().enumerate() {
        if value.fields[..index].iter().any(|(previous, _)| previous == name) {
            push_error(errors, Error::new(*span, format!("duplicate {kind} field `{name}`")));
        } else if !field_exists(fields, name) {
            push_error(errors, Error::new(*span, format!("unknown model field `{name}`")));
        }
    }
}

/// Validates duplicate field attributes, capability rules, ranges, and relation
/// paths.
fn validate_field(field: &FieldIr, fields: &[FieldIr], errors: &mut Option<Error>) {
    for span in field.opaque.iter().skip(1) {
        push_error(errors, Error::new(*span, "duplicate `opaque` attribute"));
    }
    for (index, attribute) in field.attributes.iter().enumerate() {
        for previous in &field.attributes[..index] {
            validate_field_attribute_pair(previous, attribute, errors);
        }
        if !field.opaque.is_empty() && is_shape_constraint(attribute) {
            push_error(
                errors,
                Error::new(
                    field_attribute_span(attribute),
                    format!("`opaque` cannot be combined with `{}`", field_attribute_name(attribute)),
                ),
            );
        }
        validate_field_attribute(attribute, field, fields, errors);
    }
}

/// Reports duplicate or mutually exclusive pairs of canonical field attributes.
fn validate_field_attribute_pair(previous: &FieldAttributeIr, current: &FieldAttributeIr, errors: &mut Option<Error>) {
    let previous_name = field_attribute_name(previous);
    let current_name = field_attribute_name(current);
    if previous_name == current_name {
        push_error(
            errors,
            Error::new(
                field_attribute_span(current),
                format!("duplicate `{current_name}` attribute"),
            ),
        );
    } else if matches!(previous, FieldAttributeIr::Decimal(_)) && matches!(current, FieldAttributeIr::Decimal(_)) {
        push_error(
            errors,
            Error::new(
                field_attribute_span(current),
                "`decimal` and `money` are mutually exclusive",
            ),
        );
    }
}

/// Validates one field attribute and all of its retained occurrences.
fn validate_field_attribute(
    attribute: &FieldAttributeIr,
    field: &FieldIr,
    fields: &[FieldIr],
    errors: &mut Option<Error>,
) {
    match attribute {
        FieldAttributeIr::Text(value) => validate_text(value, errors),
        FieldAttributeIr::Sequence(value) => {
            validate_duplicate_values("min_items", &value.min_items, errors);
            validate_duplicate_values("max_items", &value.max_items, errors);
            validate_duplicate_spans("unique_items", &value.unique_items, errors);
            validate_min_max("min_items", &value.min_items, "max_items", &value.max_items, errors);
            validate_sequence_shape(value, &field.ty, errors);
        }
        FieldAttributeIr::Map(value) => {
            validate_duplicate_values("min_entries", &value.min_entries, errors);
            validate_duplicate_values("max_entries", &value.max_entries, errors);
            validate_min_max(
                "min_entries",
                &value.min_entries,
                "max_entries",
                &value.max_entries,
                errors,
            );
        }
        FieldAttributeIr::Temporal(value) => {
            validate_duplicate_values("precision", &value.precision, errors);
            validate_duplicate_values("normalization", &value.normalization, errors);
        }
        FieldAttributeIr::Decimal(value) => validate_decimal(value, errors),
        FieldAttributeIr::Element(value) => validate_element(value, errors),
        FieldAttributeIr::Reference(value) => {
            validate_duplicate_literals("target", &value.target, errors);
            validate_duplicate_field_paths("target_field", &value.target_field, errors);
            validate_duplicate_values("must_exist", &value.must_exist, errors);
            validate_duplicate_field_paths("same_as", &value.same_as, errors);
            for path in &value.same_as {
                if let Some(first) = path.first()
                    && !field_exists(fields, &first.name)
                {
                    push_unknown_field(errors, first);
                }
            }
        }
        FieldAttributeIr::LookupRelation(value) => {
            validate_duplicate_type_paths("target", &value.target, errors);
            validate_duplicate_field_paths("target_field", &value.target_field, errors);
        }
        FieldAttributeIr::Codec(value) => {
            validate_duplicate_literals("codec strategy", &value.name, errors);
            validate_strategy_name("codec", &value.name, errors);
        }
        FieldAttributeIr::Generator(value) => {
            validate_duplicate_literals("generator strategy", &value.name, errors);
            validate_strategy_name("generator", &value.name, errors);
        }
    }
}

/// Validates repeated text arguments and text-length ranges.
///
/// # Parameters
///
/// - `value`: The parsed text constraint.
/// - `errors`: The combined diagnostics accumulated so far.
fn validate_text(value: &TextAttribute, errors: &mut Option<Error>) {
    validate_duplicate_values("min_chars", &value.min_chars, errors);
    validate_duplicate_values("max_chars", &value.max_chars, errors);
    validate_duplicate_values("min_bytes", &value.min_bytes, errors);
    validate_duplicate_values("max_bytes", &value.max_bytes, errors);
    validate_duplicate_values("repertoire", &value.repertoire, errors);
    validate_duplicate_spans("non_blank", &value.non_blank, errors);
    validate_duplicate_values("format", &value.format, errors);
    validate_min_max("min_chars", &value.min_chars, "max_chars", &value.max_chars, errors);
    validate_min_max("min_bytes", &value.min_bytes, "max_bytes", &value.max_bytes, errors);
}

/// Validates element constraint uniqueness and retained arguments.
///
/// # Parameters
///
/// - `value`: The normalized element metadata declaration.
/// - `errors`: The combined diagnostics accumulated so far.
fn validate_element(value: &ElementIr, errors: &mut Option<Error>) {
    for (index, attribute) in value.attributes.iter().enumerate() {
        let name = element_constraint_name(attribute);
        if value.attributes[..index]
            .iter()
            .any(|previous| element_constraint_name(previous) == name)
        {
            push_error(
                errors,
                Error::new(
                    element_constraint_span(attribute),
                    format!("duplicate `{name}` element constraint"),
                ),
            );
        }
        match attribute {
            ElementConstraintIr::Text(value) => validate_text(value, errors),
            ElementConstraintIr::Decimal(value) => validate_decimal(value, errors),
        }
    }
}

/// Returns the source name of one element constraint.
fn element_constraint_name(value: &ElementConstraintIr) -> &'static str {
    match value {
        ElementConstraintIr::Text(_) => "text",
        ElementConstraintIr::Decimal(_) => "decimal",
    }
}

/// Returns the source span of one element constraint.
fn element_constraint_span(value: &ElementConstraintIr) -> Span {
    match value {
        ElementConstraintIr::Text(value) => value.span,
        ElementConstraintIr::Decimal(value) => value.value.span,
    }
}

/// Rejects an empty logical codec or generator name.
fn validate_strategy_name(kind: &str, names: &[LitStr], errors: &mut Option<Error>) {
    if let Some(name) = names.first()
        && name.value().trim().is_empty()
    {
        push_error(
            errors,
            Error::new(name.span(), format!("`{kind}` strategy name cannot be empty")),
        );
    }
}

/// Validates repeated decimal arguments, scale, and precision semantics.
fn validate_decimal(value: &DecimalIr, errors: &mut Option<Error>) {
    validate_duplicate_values("precision", &value.value.precision, errors);
    validate_duplicate_values("scale", &value.value.scale, errors);
    validate_duplicate_values("rounding", &value.value.rounding, errors);
    if matches!(value.semantic, DecimalSemantic::Money) && value.value.scale.is_empty() {
        push_error(
            errors,
            Error::new(value.value.span, "`money` requires an explicit `scale`"),
        );
    }
    if let (Some(precision), Some(scale)) = (value.value.precision.first(), value.value.scale.first())
        && scale.value > precision.value
    {
        push_error(
            errors,
            Error::new(scale.span, "decimal `scale` cannot exceed `precision`"),
        );
    }
}

/// Validates the fixed-length-array redundancy rule when the array syntax is
/// explicit.
fn validate_sequence_shape(value: &SequenceAttribute, ty: &Type, errors: &mut Option<Error>) {
    if !is_syntactic_array(ty) {
        return;
    }
    for occurrence in &value.min_items {
        push_error(
            errors,
            Error::new(occurrence.span, "array length is fixed by its type; remove `min_items`"),
        );
    }
    for occurrence in &value.max_items {
        push_error(
            errors,
            Error::new(occurrence.span, "array length is fixed by its type; remove `max_items`"),
        );
    }
}

/// Returns whether a type is explicitly written as an array, allowing grouping
/// parentheses.
fn is_syntactic_array(ty: &Type) -> bool {
    match ty {
        Type::Array(_) => true,
        Type::Group(group) => is_syntactic_array(&group.elem),
        Type::Paren(paren) => is_syntactic_array(&paren.elem),
        _ => false,
    }
}

/// Returns the source spelling represented by normalized decimal semantics.
fn decimal_name(value: &DecimalIr) -> &'static str {
    match value.semantic {
        DecimalSemantic::Number => "decimal",
        DecimalSemantic::Money => "money",
    }
}

/// Returns whether an attribute depends on a non-opaque field shape.
fn is_shape_constraint(attribute: &FieldAttributeIr) -> bool {
    matches!(
        attribute,
        FieldAttributeIr::Text(_)
            | FieldAttributeIr::Sequence(_)
            | FieldAttributeIr::Map(_)
            | FieldAttributeIr::Temporal(_)
            | FieldAttributeIr::Decimal(_)
            | FieldAttributeIr::Element(_)
    )
}

/// Validates a pair of retained `u32` minimum and maximum occurrences.
fn validate_min_max(
    min_name: &str,
    min: &[SpannedValue<u32>],
    max_name: &str,
    max: &[SpannedValue<u32>],
    errors: &mut Option<Error>,
) {
    if let (Some(min), Some(max)) = (min.first(), max.first())
        && min.value > max.value
    {
        push_error(
            errors,
            Error::new(min.span, format!("`{min_name}` cannot exceed `{max_name}`")),
        );
    }
}

/// Reports every occurrence after the first for a single-valued spanned
/// argument.
fn validate_duplicate_values<T>(name: &str, values: &[SpannedValue<T>], errors: &mut Option<Error>) {
    for value in values.iter().skip(1) {
        push_error(errors, Error::new(value.span, format!("duplicate `{name}` argument")));
    }
}

/// Reports every marker occurrence after the first.
fn validate_duplicate_spans(name: &str, spans: &[Span], errors: &mut Option<Error>) {
    for span in spans.iter().skip(1) {
        push_error(errors, Error::new(*span, format!("duplicate `{name}` argument")));
    }
}

/// Reports every string literal occurrence after the first.
fn validate_duplicate_literals(name: &str, values: &[LitStr], errors: &mut Option<Error>) {
    for value in values.iter().skip(1) {
        push_error(errors, Error::new(value.span(), format!("duplicate `{name}` argument")));
    }
}

/// Reports every type-path occurrence after the first.
fn validate_duplicate_type_paths(name: &str, values: &[TypePath], errors: &mut Option<Error>) {
    for value in values.iter().skip(1) {
        push_error(errors, Error::new(value.span(), format!("duplicate `{name}` argument")));
    }
}

/// Reports every field-path occurrence after the first.
fn validate_duplicate_field_paths(name: &str, values: &[Vec<FieldName>], errors: &mut Option<Error>) {
    for value in values.iter().skip(1) {
        if let Some(first) = value.first() {
            push_error(errors, Error::new(first.span, format!("duplicate `{name}` argument")));
        }
    }
}

/// Returns whether the model declares a field with the normalized name.
fn field_exists(fields: &[FieldIr], name: &str) -> bool {
    fields.iter().any(|field| field.name == name)
}

/// Adds a missing-field diagnostic at the original field reference.
fn push_unknown_field(errors: &mut Option<Error>, field: &FieldName) {
    push_error(
        errors,
        Error::new(field.span, format!("unknown model field `{}`", field.name)),
    );
}

/// Returns the source name for a canonical model attribute.
fn model_attribute_name(attribute: &ModelAttributeIr) -> &'static str {
    match attribute {
        ModelAttributeIr::PrimaryKey(_) => "primary_key",
        ModelAttributeIr::Unique(_) => "unique",
        ModelAttributeIr::Index(_) => "index",
        ModelAttributeIr::Key(_) => "key",
        ModelAttributeIr::Ownership(_) => "ownership",
    }
}

/// Returns the source span for a canonical model attribute.
fn model_attribute_span(attribute: &ModelAttributeIr) -> Span {
    match attribute {
        ModelAttributeIr::PrimaryKey(value) => value.span,
        ModelAttributeIr::Unique(value) => value.span,
        ModelAttributeIr::Index(value) | ModelAttributeIr::Key(value) => value.span,
        ModelAttributeIr::Ownership(value) => value.span,
    }
}

/// Returns the source name for a canonical field attribute.
fn field_attribute_name(attribute: &FieldAttributeIr) -> &'static str {
    match attribute {
        FieldAttributeIr::Text(_) => "text",
        FieldAttributeIr::Sequence(_) => "sequence",
        FieldAttributeIr::Map(_) => "map",
        FieldAttributeIr::Temporal(_) => "time",
        FieldAttributeIr::Decimal(value) => decimal_name(value),
        FieldAttributeIr::Element(_) => "element",
        FieldAttributeIr::Reference(_) => "reference",
        FieldAttributeIr::LookupRelation(_) => "lookup_relation",
        FieldAttributeIr::Codec(_) => "codec",
        FieldAttributeIr::Generator(_) => "generator",
    }
}

/// Returns the source span for a canonical field attribute.
fn field_attribute_span(attribute: &FieldAttributeIr) -> Span {
    match attribute {
        FieldAttributeIr::Text(value) => value.span,
        FieldAttributeIr::Sequence(value) => value.span,
        FieldAttributeIr::Map(value) => value.span,
        FieldAttributeIr::Temporal(value) => value.span,
        FieldAttributeIr::Decimal(value) => value.value.span,
        FieldAttributeIr::Element(value) => value.span,
        FieldAttributeIr::Reference(value) => value.span,
        FieldAttributeIr::LookupRelation(value) => value.span,
        FieldAttributeIr::Codec(value) | FieldAttributeIr::Generator(value) => value.span,
    }
}

/// Combines one additional local diagnostic with prior diagnostics.
fn push_error(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(errors) => errors.combine(error),
        None => *errors = Some(error),
    }
}

/// Converts an optional combined diagnostic into the validator's result.
fn finish(errors: Option<Error>) -> Result<()> {
    match errors {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
