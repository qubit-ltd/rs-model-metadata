// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalization from parsed attribute syntax to expansion-ready semantic IR.

use proc_macro2::Span;
use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;

use super::DecimalIr;
use super::DecimalSemantic;
use super::ElementConstraintIr;
use super::ElementIr;
use super::FieldAttributeIr;
use super::FieldIr;
use super::ModelAttributeIr;
use super::ModelIr;
use super::ModelShapeIr;
use super::NamedFieldsIr;
use super::OwnershipIr;
use super::PrimaryKeyFieldIr;
use super::PrimaryKeyIr;
use super::UniqueFieldIr;
use super::UniqueIr;
use crate::attribute;
use crate::attribute::ElementAttribute;
use crate::attribute::ElementConstraintAttribute;
use crate::attribute::FieldAttribute;
use crate::attribute::FieldName;
use crate::attribute::ModelAttribute;
use crate::input::ModelField;
use crate::input::ModelInput;
use crate::input::ModelShape;

/// Normalizes a parsed model into the sole semantic representation consumed by
/// expansion.
///
/// Field `identifier`, `unique`, and `index` shorthands become model
/// attributes. Monetary values become decimal IR with money semantics. The
/// function preserves source spans for later local validation and diagnostics.
///
/// # Parameters
///
/// - `input`: The parsed model declaration to normalize.
///
/// # Returns
///
/// Returns the canonical intermediate representation used by validation and
/// token expansion.
pub(crate) fn normalize(input: ModelInput) -> ModelIr {
    let ModelInput {
        ident,
        id,
        attributes,
        shape,
    } = input;
    let textual = attributes
        .iter()
        .any(|attribute| matches!(attribute, ModelAttribute::Textual));
    let model_attribute_count = attributes.len() - usize::from(textual);
    let mut model_attributes = attributes
        .into_iter()
        .filter_map(normalize_model_attribute)
        .collect::<Vec<_>>();
    let shape = match shape {
        ModelShape::NamedStruct(fields) => {
            let (fields, shorthand) = normalize_fields(fields);
            model_attributes.extend(shorthand);
            ModelShapeIr::NamedStruct(fields)
        }
        ModelShape::UnitStruct => ModelShapeIr::UnitStruct,
        ModelShape::Newtype(field) => {
            let (field, shorthand) = normalize_field(*field);
            model_attributes.extend(shorthand);
            ModelShapeIr::Newtype(Box::new(field))
        }
        ModelShape::FieldlessEnum(variants) => ModelShapeIr::FieldlessEnum(variants),
    };

    ModelIr {
        ident,
        id,
        attributes: model_attributes,
        model_attribute_count,
        textual,
        shape,
    }
}

/// Converts one parsed model attribute to canonical IR.
fn normalize_model_attribute(attribute: ModelAttribute) -> Option<ModelAttributeIr> {
    match attribute {
        ModelAttribute::Textual => None,
        ModelAttribute::PrimaryKey(attribute) => {
            let fields = attribute
                .fields
                .into_iter()
                .map(|field| PrimaryKeyFieldIr {
                    name: field.name,
                    span: field.span,
                })
                .collect();
            Some(ModelAttributeIr::PrimaryKey(PrimaryKeyIr {
                fields,
                generated: attribute.generated,
                span: attribute.span,
            }))
        }
        ModelAttribute::Index(attribute) => Some(ModelAttributeIr::Index(normalize_named_fields(attribute))),
        ModelAttribute::Key(attribute) => Some(ModelAttributeIr::Key(normalize_named_fields(attribute))),
        ModelAttribute::Ownership(attribute) => Some(ModelAttributeIr::Ownership(OwnershipIr {
            owner: attribute.owner,
            span: attribute.span,
        })),
    }
}

/// Converts parsed named-field syntax to canonical IR.
fn normalize_named_fields(attribute: attribute::NamedFieldsAttribute) -> NamedFieldsIr {
    NamedFieldsIr {
        name: attribute.name,
        fields: attribute
            .fields
            .into_iter()
            .map(|field| (field.name, field.span))
            .collect(),
        span: attribute.span,
        implicit: false,
    }
}

/// Normalizes named fields and combines identifier shorthands into one primary
/// key.
fn normalize_fields(fields: Vec<ModelField>) -> (Vec<FieldIr>, Vec<ModelAttributeIr>) {
    let mut normalized = Vec::with_capacity(fields.len());
    let mut shorthand = Vec::new();
    let mut identifier_fields = Vec::new();
    let mut generated_fields = Vec::new();
    for field in fields {
        let (field, mut attributes) = normalize_field(field);
        for attribute in attributes.drain(..) {
            match attribute {
                ModelAttributeIr::PrimaryKey(primary_key) => {
                    identifier_fields.extend(primary_key.fields);
                    generated_fields.extend(primary_key.generated);
                }
                other => shorthand.push(other),
            }
        }
        normalized.push(field);
    }
    if let Some(first) = identifier_fields.first() {
        shorthand.insert(
            0,
            ModelAttributeIr::PrimaryKey(PrimaryKeyIr {
                span: first.span,
                fields: identifier_fields,
                generated: generated_fields,
            }),
        );
    }
    (normalized, shorthand)
}

/// Normalizes one field and returns model attributes produced by its
/// shorthands.
fn normalize_field(field: ModelField) -> (FieldIr, Vec<ModelAttributeIr>) {
    let ModelField {
        ordinal,
        name,
        ty,
        attributes,
    } = field;
    let mut field_attributes = Vec::new();
    let mut model_attributes = Vec::new();
    let mut opaque = Vec::new();
    let has_text_attribute = attributes
        .iter()
        .any(|attribute| matches!(attribute, FieldAttribute::Text(_)));
    for attribute in attributes {
        match attribute {
            FieldAttribute::Identifier(attribute) => {
                let generated = attribute
                    .generated
                    .into_iter()
                    .map(|span| FieldName {
                        name: name.clone(),
                        span,
                    })
                    .collect();
                model_attributes.push(ModelAttributeIr::PrimaryKey(PrimaryKeyIr {
                    fields: vec![PrimaryKeyFieldIr {
                        name: name.clone(),
                        span: attribute.span,
                    }],
                    generated,
                    span: attribute.span,
                }));
            }
            FieldAttribute::Unique(attribute) => {
                let explicit_ignore_case = attribute.ignore_case_values.first().map(|value| value.value);
                let default_ignore_case =
                    explicit_ignore_case != Some(false) && (is_string_type(&ty) || has_text_attribute);
                let ignore_case =
                    if explicit_ignore_case == Some(true) || attribute.legacy_ignore_case || default_ignore_case {
                        vec![FieldName {
                            name: name.clone(),
                            span: attribute.span,
                        }]
                    } else {
                        Vec::new()
                    };
                let mut fields = attribute.respect_to;
                fields.push(FieldName {
                    name: name.clone(),
                    span: attribute.span,
                });
                model_attributes.push(ModelAttributeIr::Unique(UniqueIr {
                    name: attribute.name,
                    fields: fields
                        .into_iter()
                        .map(|field| UniqueFieldIr {
                            name: field.name,
                            span: field.span,
                        })
                        .collect(),
                    ignore_case,
                    span: attribute.span,
                }));
            }
            FieldAttribute::Index(span) => {
                model_attributes.push(ModelAttributeIr::Index(NamedFieldsIr {
                    name: Vec::new(),
                    fields: vec![(name.clone(), span)],
                    span,
                    implicit: false,
                }));
            }
            FieldAttribute::Text(attribute) => {
                field_attributes.push(FieldAttributeIr::Text(attribute));
            }
            FieldAttribute::Sequence(attribute) => {
                field_attributes.push(FieldAttributeIr::Sequence(attribute));
            }
            FieldAttribute::Map(attribute) => {
                field_attributes.push(FieldAttributeIr::Map(attribute));
            }
            FieldAttribute::Temporal(attribute) => {
                field_attributes.push(FieldAttributeIr::Temporal(attribute));
            }
            FieldAttribute::Decimal(value) => {
                field_attributes.push(FieldAttributeIr::Decimal(DecimalIr {
                    value,
                    semantic: DecimalSemantic::Number,
                }));
            }
            FieldAttribute::Money(value) => {
                field_attributes.push(FieldAttributeIr::Decimal(DecimalIr {
                    value,
                    semantic: DecimalSemantic::Money,
                }));
            }
            FieldAttribute::Element(value) => {
                field_attributes.push(FieldAttributeIr::Element(normalize_element(value)));
            }
            FieldAttribute::Reference(attribute) => {
                field_attributes.push(FieldAttributeIr::Reference(attribute));
                model_attributes.push(ModelAttributeIr::Index(NamedFieldsIr {
                    name: Vec::new(),
                    fields: vec![(name.clone(), Span::call_site())],
                    span: Span::call_site(),
                    implicit: true,
                }));
            }
            FieldAttribute::LookupRelation(attribute) => {
                field_attributes.push(FieldAttributeIr::LookupRelation(attribute));
            }
            FieldAttribute::Codec(attribute) => {
                field_attributes.push(FieldAttributeIr::Codec(attribute));
            }
            FieldAttribute::Generator(attribute) => {
                field_attributes.push(FieldAttributeIr::Generator(attribute));
            }
            FieldAttribute::Opaque(span) => opaque.push(span),
            FieldAttribute::KeepSerializing => {}
        }
    }

    (
        FieldIr {
            ordinal,
            name,
            ty,
            attributes: field_attributes,
            opaque,
        },
        model_attributes,
    )
}

/// Returns whether a field is syntactically a `String` or `Option<String>`.
fn is_string_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            let Some(segment) = path.path.segments.last() else {
                return false;
            };
            if segment.ident == "String" {
                return true;
            }
            if segment.ident != "Option" {
                return false;
            }
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            matches!(
                arguments.args.first(),
                Some(GenericArgument::Type(Type::Path(inner)))
                    if inner.path.segments.last().is_some_and(|segment| segment.ident == "String")
            )
        }
        Type::Group(group) => is_string_type(&group.elem),
        Type::Paren(paren) => is_string_type(&paren.elem),
        _ => false,
    }
}

/// Normalizes element constraints into their runtime semantic forms.
///
/// # Parameters
///
/// - `value`: The parsed element constraint syntax.
///
/// # Returns
///
/// Element IR containing text and ordinary decimal constraints.
fn normalize_element(value: ElementAttribute) -> ElementIr {
    let attributes = value
        .attributes
        .into_iter()
        .map(|attribute| match attribute {
            ElementConstraintAttribute::Text(value) => ElementConstraintIr::Text(value),
            ElementConstraintAttribute::Decimal(value) => ElementConstraintIr::Decimal(DecimalIr {
                value,
                semantic: DecimalSemantic::Number,
            }),
        })
        .collect();
    ElementIr {
        attributes,
        span: value.span,
    }
}
