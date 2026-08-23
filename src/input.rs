// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
// qubit-style: allow multiple-public-types
//! Parsed model declarations supported by the derive macro.

use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Field;
use syn::Fields;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Type;
use syn::Variant;
use syn::ext::IdentExt;
use syn::spanned::Spanned;

use super::attribute::FieldAttribute;
use super::attribute::ModelAttribute;
use super::attribute::parse_field_attributes;
use super::attribute::parse_model_attributes;
use super::attribute_support::serialized_variant_name;

/// The parsed input required to generate model metadata.
pub(crate) struct ModelInput {
    /// The name of the declared model type.
    pub(crate) ident: Ident,
    /// Raw stable model-ID literals in source order.
    pub(crate) id: Vec<LitStr>,
    /// Parsed model-level attributes in source order.
    pub(crate) attributes: Vec<ModelAttribute>,
    /// The supported structural form of the model.
    pub(crate) shape: ModelShape,
}

/// A supported structural form of a model declaration.
pub(crate) enum ModelShape {
    /// A struct with named fields in declaration order.
    NamedStruct(Vec<ModelField>),
    /// A struct with no fields.
    UnitStruct,
    /// A tuple struct with exactly one field.
    Newtype(Box<ModelField>),
    /// An enum whose variants all have no fields.
    FieldlessEnum(Vec<ModelVariant>),
}

/// A declared model field.
pub(crate) struct ModelField {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized field name.
    pub(crate) name: String,
    /// The declared Rust field type.
    pub(crate) ty: Type,
    /// Parsed field-level attributes in source order.
    pub(crate) attributes: Vec<FieldAttribute>,
}

/// A fieldless enum variant.
pub(crate) struct ModelVariant {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized variant name.
    pub(crate) name: String,
}

impl ModelInput {
    /// Parses a derive input into one of the declaration shapes supported by
    /// this macro.
    ///
    /// Returns an error at the unsupported declaration's span when the input is
    /// a union, a tuple struct with any field count other than one, an enum
    /// variant with fields, or a generic model.
    pub(crate) fn parse(input: DeriveInput) -> Result<Self> {
        let mut errors = None;
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            combine_error(
                &mut errors,
                Error::new(input.generics.span(), "Model derive does not support generic models"),
            );
        }
        let attributes = match parse_model_attributes(&input.attrs) {
            Ok(attributes) => Some(attributes),
            Err(error) => {
                combine_error(&mut errors, error);
                None
            }
        };
        let ident = input.ident;
        let shape = match match input.data {
            Data::Struct(data) => Self::parse_struct(data.fields),
            Data::Enum(data) => Self::parse_enum(data.variants.into_iter().collect()),
            Data::Union(data) => Err(Error::new_spanned(
                data.union_token,
                "Model derive does not support unions",
            )),
        } {
            Ok(shape) => Some(shape),
            Err(error) => {
                combine_error(&mut errors, error);
                None
            }
        };

        if let Some(error) = errors {
            return Err(error);
        }
        let (Some(attributes), Some(shape)) = (attributes, shape) else {
            return Err(Error::new(
                ident.span(),
                "Model derive input parsing did not produce a model",
            ));
        };

        Ok(Self {
            ident,
            id: attributes.id,
            attributes: attributes.attributes,
            shape,
        })
    }

    /// Parses supported struct forms and reports unsupported tuple struct
    /// arities.
    fn parse_struct(fields: Fields) -> Result<ModelShape> {
        match fields {
            Fields::Named(fields) => {
                let mut parsed = Vec::with_capacity(fields.named.len());
                let mut errors = None;
                for (ordinal, field) in fields.named.into_iter().enumerate() {
                    match Self::parse_named_field(ordinal, field) {
                        Ok(field) => parsed.push(field),
                        Err(error) => combine_error(&mut errors, error),
                    }
                }
                match errors {
                    Some(error) => Err(error),
                    None => Ok(ModelShape::NamedStruct(parsed)),
                }
            }
            Fields::Unit => Ok(ModelShape::UnitStruct),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let span = fields.span();
                let field = fields
                    .unnamed
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::new(span, "tuple newtype field is missing"))?;
                Ok(ModelShape::Newtype(Box::new(ModelField {
                    ordinal: 0,
                    name: "0".to_owned(),
                    ty: field.ty,
                    attributes: parse_field_attributes(&field.attrs)?,
                })))
            }
            Fields::Unnamed(fields) => Err(Error::new_spanned(
                fields,
                "Model derive only supports single-field tuple newtypes",
            )),
        }
    }

    /// Converts one syntactically named field into its minimal metadata input.
    fn parse_named_field(ordinal: usize, field: Field) -> Result<ModelField> {
        let span = field.span();
        let attributes = parse_field_attributes(&field.attrs)?;
        let name = field
            .ident
            .ok_or_else(|| Error::new(span, "named struct field is missing an identifier"))?
            .unraw()
            .to_string();

        Ok(ModelField {
            ordinal,
            name,
            ty: field.ty,
            attributes,
        })
    }

    /// Parses a fieldless enum and combines unsupported fields and variant
    /// attributes.
    fn parse_enum(variants: Vec<Variant>) -> Result<ModelShape> {
        let mut errors: Option<Error> = None;
        for variant in &variants {
            for attribute in variant
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("model"))
            {
                let error = Error::new_spanned(attribute, "`model` attributes are not supported on enum variants");
                combine_error(&mut errors, error);
            }
            if !matches!(variant.fields, Fields::Unit) {
                let error = Error::new(variant.fields.span(), "Model derive only supports fieldless enums");
                combine_error(&mut errors, error);
            }
        }
        if let Some(error) = errors {
            return Err(error);
        }

        Ok(ModelShape::FieldlessEnum(
            variants
                .into_iter()
                .enumerate()
                .map(|(ordinal, variant)| {
                    let name = serialized_variant_name(&variant)?;
                    Ok(ModelVariant { ordinal, name })
                })
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

/// Combines one input diagnostic with any diagnostics already collected.
fn combine_error(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(errors) => errors.combine(error),
        None => *errors = Some(error),
    }
}
