// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

fn validate_declaration_ir(declaration: &DeclarationIr, item: &DeriveInput) -> Result<()> {
    let mut errors = None;
    let options = &declaration.options;
    if options.source_id.is_some() && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "source_id is only valid for Projection"),
        );
    }
    if options.source.is_some() && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "source is only valid for Projection"),
        );
    }
    if options.source.is_some() && options.source_id.is_some() {
        combine(
            &mut errors,
            Error::new_spanned(
                &item.ident,
                "Projection accepts only one of `source` or `source_id`",
            ),
        );
    }
    if options.open && (options.source.is_some() || options.source_id.is_some()) {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "open Projection cannot declare a fixed source"),
        );
    }
    if options.open && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "open is only valid for Projection"),
        );
    }
    if options.transparent && declaration.kind != MacroKind::Value {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "transparent is only valid for Value"),
        );
    }
    if options.codec.is_some() && declaration.kind != MacroKind::Value {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "canonical codec is only valid for Value"),
        );
    }
    if options.transparent && declaration.fields.len() != 1 {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "transparent Value requires exactly one field"),
        );
    }
    if options.no_copy && declaration.kind != MacroKind::Enum {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "no_copy is only valid for Enum"),
        );
    }
    if options.no_copy && options.copy {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "copy and no_copy cannot be combined"),
        );
    }
    let all_fields: Vec<_> = declaration
        .fields
        .iter()
        .chain(
            declaration
                .variants
                .iter()
                .flat_map(|variant| &variant.fields),
        )
        .collect();
    if options.no_redact
        && all_fields.iter().any(|field| {
            field.occurrences.iter().any(|value| {
                matches!(
                    value,
                    FieldOccurrence::Redact(_)
                        | FieldOccurrence::Selector(SelectorIr {
                            redact: Some(_),
                            ..
                        })
                )
            })
        })
    {
        combine(
            &mut errors,
            Error::new_spanned(
                &item.ident,
                "no_redact cannot be combined with field redaction rules",
            ),
        );
    }
    for field in &all_fields {
        let has_identifier = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Identifier(_)));
        let has_reference = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Reference(_)));
        if has_identifier && !matches!(declaration.kind, MacroKind::Entity | MacroKind::Projection)
        {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "identifier is only valid for Entity and Projection",
                ),
            );
        }
        if has_reference && matches!(declaration.kind, MacroKind::Enum | MacroKind::Value) {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "Enum and Value fields cannot declare references",
                ),
            );
        }
        let has_implicit_index = field.occurrences.iter().any(|value| {
            matches!(
                value,
                FieldOccurrence::Identifier(_)
                    | FieldOccurrence::Unique(_)
                    | FieldOccurrence::Reference(_)
            )
        });
        if has_implicit_index
            && field
                .occurrences
                .iter()
                .any(|value| matches!(value, FieldOccurrence::Indexed))
        {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "explicit indexed is redundant with identifier, unique, or reference",
                ),
            );
        }
        if field.keep_serializing && (!field.named || omission_kind(&field.ty).is_none()) {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "keep_serializing requires a named Option or standard collection field",
                ),
            );
        }
        let predicates: [fn(&FieldOccurrence) -> bool; 6] = [
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Unique(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Reference(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::KeyPart(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Codec(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Redact(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Serde(_)),
        ];
        for predicate in predicates {
            if field
                .occurrences
                .iter()
                .filter(|value| predicate(value))
                .count()
                > 1
            {
                combine(
                    &mut errors,
                    Error::new_spanned(&item.ident, "duplicate singleton field declaration"),
                );
            }
        }
        let opaque = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Opaque));
        if opaque
            && field
                .occurrences
                .iter()
                .any(|value| matches!(value, FieldOccurrence::Reference(_)))
        {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "opaque cannot be combined with reference"),
            );
        }
        for position in [
            SelectorPositionIr::Element,
            SelectorPositionIr::MapKey,
            SelectorPositionIr::MapValue,
        ] {
            if field
                .occurrences
                .iter()
                .filter(|value| matches!(value, FieldOccurrence::Selector(selector) if selector.position == position))
                .count()
                > 1
            {
                combine(
                    &mut errors,
                    Error::new_spanned(&item.ident, "duplicate selector position"),
                );
            }
        }
        let field_redact = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Redact(_)));
        let selector_redact = field.occurrences.iter().any(|value| {
            matches!(
                value,
                FieldOccurrence::Selector(SelectorIr {
                    redact: Some(_),
                    ..
                })
            )
        });
        if field_redact && selector_redact {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "field and selector redaction cannot overlap"),
            );
        }
        for selector in field.occurrences.iter().filter_map(|value| match value {
            FieldOccurrence::Selector(selector) => Some(selector),
            _ => None,
        }) {
            if let Some(redact) = &selector.redact
                && !matches!(redact.mode, RedactModeIr::Level(_))
            {
                combine(
                    &mut errors,
                    Error::new_spanned(
                        &item.ident,
                        "selector redaction currently supports only `redact(level = \"...\")`; `skip` is forbidden",
                    ),
                );
            }
        }
        validate_field_constraints(field, &mut errors);
    }
    let mut orders: Vec<_> = declaration
        .fields
        .iter()
        .filter_map(|field| {
            field.occurrences.iter().find_map(|value| match value {
                FieldOccurrence::KeyPart(order) => Some(*order),
                _ => None,
            })
        })
        .collect();
    orders.sort_unstable();
    if orders
        .iter()
        .copied()
        .enumerate()
        .any(|(expected, actual)| expected != actual)
    {
        combine(
            &mut errors,
            Error::new_spanned(
                &item.ident,
                "key_part orders must be unique and contiguous from zero",
            ),
        );
    }
    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(())
    }
}

/// Adds implicit container constraints required by selector metadata.
fn normalize_selector_containers(field: &mut FieldIr) {
    let has_element = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::Element,
                ..
            })
        )
    });
    let has_map_selector = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::MapKey | SelectorPositionIr::MapValue,
                ..
            })
        )
    });
    if has_element
        && !field.occurrences.iter().any(|value| {
            matches!(
                value,
                FieldOccurrence::Constraint(ConstraintIr::Sequence { .. })
            )
        })
    {
        field
            .occurrences
            .push(FieldOccurrence::Constraint(ConstraintIr::Sequence {
                min: None,
                max: None,
                unique: false,
            }));
    }
    if has_map_selector
        && !field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Constraint(ConstraintIr::Map { .. })))
    {
        field
            .occurrences
            .push(FieldOccurrence::Constraint(ConstraintIr::Map {
                min: None,
                max: None,
            }));
    }
}

/// Validates that a field's constraints match its Rust type and role.
fn validate_field_constraints(field: &FieldIr, errors: &mut Option<Error>) {
    let mut kinds = std::collections::HashSet::new();
    for constraint in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Constraint(value) => Some(value),
        _ => None,
    }) {
        let kind = match constraint {
            ConstraintIr::Text(_) => "text",
            ConstraintIr::Decimal(value) if value.money => "decimal-or-money",
            ConstraintIr::Decimal(_) => "decimal-or-money",
            ConstraintIr::Time(_) => "time",
            ConstraintIr::Sequence { .. } => "sequence",
            ConstraintIr::Map { .. } => "map",
        };
        if !kinds.insert(kind) {
            combine(
                errors,
                Error::new(field.span, format!("duplicate {kind} constraint")),
            );
        }
    }
    for selector in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Selector(value) => Some(value),
        _ => None,
    }) {
        let mut selector_kinds = std::collections::HashSet::new();
        for constraint in &selector.constraints {
            let kind = match constraint {
                ConstraintIr::Text(_) => "text",
                ConstraintIr::Decimal(_) => "decimal-or-money",
                ConstraintIr::Time(_) => "time",
                ConstraintIr::Sequence { .. } | ConstraintIr::Map { .. } => "container",
            };
            if !selector_kinds.insert(kind) {
                combine(
                    errors,
                    Error::new(field.span, format!("duplicate selector {kind} constraint")),
                );
            }
        }
    }

    for constraint in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Constraint(value) => Some(value),
        _ => None,
    }) {
        match constraint {
            ConstraintIr::Sequence { min, max, .. } => {
                if min.zip(*max).is_some_and(|(min, max)| min > max) {
                    combine(
                        errors,
                        Error::new(field.span, "sequence min_items cannot exceed max_items"),
                    );
                }
            }
            ConstraintIr::Map { min, max }
                if min.zip(*max).is_some_and(|(min, max)| min > max) => {
                    combine(
                        errors,
                        Error::new(field.span, "map min_entries cannot exceed max_entries"),
                    );
                }
            _ => {}
        }
    }
}
