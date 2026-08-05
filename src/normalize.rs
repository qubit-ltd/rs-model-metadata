// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Normalization from parsed attribute syntax to expansion-ready semantic IR.

use proc_macro2::Span;
use syn::{Ident, LitStr, Type, TypePath};

use super::attribute::{
    self, DecimalAttribute, ElementAttribute, ElementConstraintAttribute, FieldAttribute,
    FieldName, LookupRelationAttribute, MapAttribute, ModelAttribute, ReferenceAttribute,
    SensitiveAttribute, SequenceAttribute, StrategyAttribute, TemporalAttribute, TextAttribute,
};
use super::input::{ModelField, ModelInput, ModelShape, ModelVariant};

/// An expansion-ready model with all shorthand syntax removed.
pub(crate) struct ModelIr {
    /// The declared model type name.
    pub(crate) ident: Ident,
    /// Canonical model-level attributes.
    pub(crate) attributes: Vec<ModelAttributeIr>,
    /// Number of attributes declared directly on the model before field
    /// shorthands were appended.
    pub(crate) model_attribute_count: usize,
    /// Whether this named model is a textual value object.
    pub(crate) textual: bool,
    /// The model's supported structural form.
    pub(crate) shape: ModelShapeIr,
}

/// A supported model shape containing normalized fields.
pub(crate) enum ModelShapeIr {
    /// A struct with named fields in declaration order.
    NamedStruct(Vec<FieldIr>),
    /// A struct with no fields.
    UnitStruct,
    /// A tuple struct with exactly one field.
    Newtype(Box<FieldIr>),
    /// An enum whose variants all have no fields.
    FieldlessEnum(Vec<ModelVariant>),
}

/// A field whose attributes have been normalized to runtime semantics.
pub(crate) struct FieldIr {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized field name.
    pub(crate) name: String,
    /// The declared Rust type.
    pub(crate) ty: Type,
    /// Canonical field-level attributes.
    pub(crate) attributes: Vec<FieldAttributeIr>,
    /// Every marker span requiring the field type to be treated as opaque.
    pub(crate) opaque: Vec<Span>,
}

/// A canonical model-level attribute.
pub(crate) enum ModelAttributeIr {
    /// The model's primary-key definition.
    PrimaryKey(PrimaryKeyIr),
    /// A unique-constraint definition.
    Unique(UniqueIr),
    /// An index definition.
    Index(NamedFieldsIr),
    /// A logical-key definition.
    Key(NamedFieldsIr),
    /// An ownership relation.
    Ownership(OwnershipIr),
}

/// A canonical primary-key definition.
pub(crate) struct PrimaryKeyIr {
    /// Key fields in declaration order.
    pub(crate) fields: Vec<PrimaryKeyFieldIr>,
    /// Every `generated(...)` field reference in source order, including
    /// invalid or duplicate ones.
    pub(crate) generated: Vec<FieldName>,
    /// The originating attribute span.
    pub(crate) span: Span,
}

/// A canonical primary-key field.
pub(crate) struct PrimaryKeyFieldIr {
    /// The normalized field name.
    pub(crate) name: String,
    /// The originating field-name or shorthand span.
    pub(crate) span: Span,
}

/// A canonical unique-constraint definition.
pub(crate) struct UniqueIr {
    /// Logical-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// Fields in comparison order.
    pub(crate) fields: Vec<UniqueFieldIr>,
    /// Every `ignore_case(...)` field reference, including invalid or
    /// duplicate ones.
    pub(crate) ignore_case: Vec<FieldName>,
    /// The originating attribute span.
    pub(crate) span: Span,
}

/// A canonical unique-constraint field.
pub(crate) struct UniqueFieldIr {
    /// The normalized field name.
    pub(crate) name: String,
    /// The originating field-name or shorthand span.
    pub(crate) span: Span,
}

/// A canonical named ordered-field declaration.
pub(crate) struct NamedFieldsIr {
    /// Logical-name occurrences in source order.
    pub(crate) name: Vec<LitStr>,
    /// Ordered normalized field names and their spans.
    pub(crate) fields: Vec<(String, Span)>,
    /// The originating attribute span.
    pub(crate) span: Span,
}

/// A canonical ownership relation.
pub(crate) struct OwnershipIr {
    /// Owning-model occurrences in source order.
    pub(crate) owner: Vec<TypePath>,
    /// The originating attribute span.
    pub(crate) span: Span,
}

/// A canonical field-level attribute emitted into `FieldMetadata`.
pub(crate) enum FieldAttributeIr {
    /// Text constraints.
    Text(TextAttribute),
    /// Ordered-sequence constraints.
    Sequence(SequenceAttribute),
    /// Map constraints.
    Map(MapAttribute),
    /// Temporal constraints.
    Temporal(TemporalAttribute),
    /// Decimal constraints with normalized domain semantics.
    Decimal(DecimalIr),
    /// Constraints applied to sequence elements.
    Element(ElementIr),
    /// A direct model reference.
    Reference(ReferenceAttribute),
    /// A lookup relation to another model.
    LookupRelation(LookupRelationAttribute),
    /// Sensitive-data handling.
    Sensitive(SensitiveAttribute),
    /// A codec strategy.
    Codec(StrategyAttribute),
    /// A generator strategy.
    Generator(StrategyAttribute),
}

/// Canonical decimal semantics shared by `decimal` and `money` syntax.
pub(crate) struct DecimalIr {
    /// Parsed decimal values.
    pub(crate) value: DecimalAttribute,
    /// Whether the value is an ordinary number or money.
    pub(crate) semantic: DecimalSemantic,
}

/// Canonical constraints applied to sequence elements.
pub(crate) struct ElementIr {
    /// Element constraints in source order.
    pub(crate) attributes: Vec<ElementConstraintIr>,
    /// The originating attribute span.
    pub(crate) span: Span,
}

/// A normalized constraint supported on migrated collection elements.
pub(crate) enum ElementConstraintIr {
    /// Text constraints for string elements.
    Text(TextAttribute),
    /// Ordinary decimal constraints for high-precision numeric elements.
    Decimal(DecimalIr),
}

/// The domain meaning of a decimal field.
pub(crate) enum DecimalSemantic {
    /// A general-purpose decimal number.
    Number,
    /// A monetary amount.
    Money,
}

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
        ModelAttribute::Unique(attribute) => {
            let fields = attribute
                .fields
                .into_iter()
                .map(|field| UniqueFieldIr {
                    name: field.name,
                    span: field.span,
                })
                .collect();
            Some(ModelAttributeIr::Unique(UniqueIr {
                name: attribute.name,
                fields,
                ignore_case: attribute.ignore_case,
                span: attribute.span,
            }))
        }
        ModelAttribute::Index(attribute) => {
            Some(ModelAttributeIr::Index(normalize_named_fields(attribute)))
        }
        ModelAttribute::Key(attribute) => {
            Some(ModelAttributeIr::Key(normalize_named_fields(attribute)))
        }
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
                let ignore_case = attribute
                    .ignore_case
                    .into_iter()
                    .map(|span| FieldName {
                        name: name.clone(),
                        span,
                    })
                    .collect();
                model_attributes.push(ModelAttributeIr::Unique(UniqueIr {
                    name: Vec::new(),
                    fields: vec![UniqueFieldIr {
                        name: name.clone(),
                        span: attribute.span,
                    }],
                    ignore_case,
                    span: attribute.span,
                }));
            }
            FieldAttribute::Index(span) => {
                model_attributes.push(ModelAttributeIr::Index(NamedFieldsIr {
                    name: Vec::new(),
                    fields: vec![(name.clone(), span)],
                    span,
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
            }
            FieldAttribute::LookupRelation(attribute) => {
                field_attributes.push(FieldAttributeIr::LookupRelation(attribute));
            }
            FieldAttribute::Sensitive(attribute) => {
                field_attributes.push(FieldAttributeIr::Sensitive(attribute));
            }
            FieldAttribute::Codec(attribute) => {
                field_attributes.push(FieldAttributeIr::Codec(attribute));
            }
            FieldAttribute::Generator(attribute) => {
                field_attributes.push(FieldAttributeIr::Generator(attribute));
            }
            FieldAttribute::Opaque(span) => opaque.push(span),
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
