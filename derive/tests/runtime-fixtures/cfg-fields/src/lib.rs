use model_runtime::__private::TypeDescriptor;
use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::{Enum, Model, Value};
use std::fmt::Display;
use std::marker::PhantomData;

// Native Rust control: cfg removes the unknown type before type checking.
#[qubit_reflect::Reflect]
struct Native {
    visible: String,
    #[cfg(any())]
    absent: MissingType,
}

#[Model]
struct CfgProbe {
    visible: String,
    #[cfg(any())]
    absent: MissingType,
    #[cfg(feature = "field-enabled")]
    active: u32,
    #[cfg_attr(not(feature = "via-cfg-attr"), cfg(any()))]
    indirect: bool,
}

#[Enum]
#[cfg(all(feature = "test-extra", feature = "enum-probe"))]
enum CfgEnum {
    #[cfg(any())]
    Gone(MissingType),
    Data {
        #[cfg(feature = "field-enabled")]
        extra: u32,
        payload: String,
    },
    Pair(#[cfg(feature = "field-enabled")] u32, String),
}

#[Enum]
#[cfg(all(feature = "test-extra", feature = "enum-probe"))]
enum GenericCfgEnum<T> {
    #[cfg(any())]
    Gone(MissingType),
    Data(
        #[cfg(any())] MissingType,
        T,
        #[cfg_attr(not(feature = "via-cfg-attr"), cfg(any()))] T,
    ),
}

#[qubit_reflect::Reflect]
struct NoTraitType;

#[qubit_reflect::Reflect]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct NoDisplayType;

#[qubit_reflect::Reflect]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct DisplayType;

impl Display for DisplayType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("display")
    }
}

#[Enum(no_redact)]
#[cfg(feature = "test-extra")]
enum ConditionalBound<T: qubit_reflect::descriptor::Reflect> {
    #[cfg(feature = "bound-enabled")]
    Payload(T),
    Empty(#[reflect(skip)] PhantomData<T>),
}

#[Enum(no_redact)]
#[cfg(feature = "test-extra")]
enum ZeroFieldPayload<T: qubit_reflect::descriptor::Reflect> {
    Payload(
        #[cfg(feature = "zero-field")]
        u64,
        #[cfg(feature = "zero-field")]
        u32,
    ),
    Empty(#[reflect(skip)] PhantomData<T>),
}

#[Value]
#[cfg(feature = "test-extra")]
struct CfgTuple(String, #[cfg(any())] MissingType);

#[Value(default)]
#[cfg(feature = "test-extra")]
struct GenericTuple<T>(#[cfg(any())] MissingType, T);

#[Value(no_redact, no_serialize, no_deserialize, transparent)]
#[cfg(feature = "test-extra")]
struct GenericTransparent<T>(#[cfg(any())] MissingType, T);

#[Value(no_redact, no_serialize, no_deserialize, transparent)]
#[cfg(feature = "test-extra")]
struct ConditionalTransparent(
    #[cfg(feature = "transparent-disabled")]
    NoDisplayType,
    #[cfg(not(feature = "transparent-disabled"))]
    DisplayType,
);

#[Value(default, no_redact, no_serialize, no_deserialize)]
#[cfg(feature = "test-extra")]
struct ConditionalDefault<T>(#[cfg(any())] T, Option<T>);

#[cfg(feature = "expect-bound-failure")]
fn active_conditional_bound_requires_clone() {
    let value = ConditionalBound::<NoTraitType>::Payload(NoTraitType);
    let _ = value.clone();
}

#[cfg(feature = "expect-transparent-failure")]
fn active_transparent_field_requires_display() {
    let _ = format!("{}", ConditionalTransparent(NoDisplayType));
}

#[cfg(feature = "tuple-limit")]
#[Enum(no_redact)]
enum TooManyTupleConditions<T: qubit_reflect::descriptor::Reflect> {
    Payload(
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
        #[cfg(feature = "tuple-limit")] u8,
    ),
    Empty(#[reflect(skip)] PhantomData<T>),
}

#[Model]
#[cfg(feature = "test-extra")]
struct Generic<T> {
    #[cfg(any())]
    absent: MissingType,
    value: T,
}

#[cfg(feature = "invalid-enabled")]
#[Value]
struct InvalidWhenEnabled(String, #[cfg(feature = "invalid-enabled")] String);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fields_match_reflection() {
        let _ = Native {
            visible: String::new(),
        };
        let metadata = TypeMetadata::of::<CfgProbe>();
        let descriptor = TypeDescriptor::of::<CfgProbe>();
        assert_eq!(metadata.fields().len(), descriptor.fields().len());
        assert_eq!(metadata.fields().first().unwrap().name(), Some("visible"));
        assert_eq!(descriptor.field("visible").unwrap().index(), 0);
        assert_eq!(
            metadata.fields().len(),
            1 + usize::from(cfg!(feature = "field-enabled"))
                + usize::from(cfg!(feature = "via-cfg-attr"))
        );
        for (index, field) in metadata.fields().iter().enumerate() {
            assert_eq!(
                descriptor.fields().get_at(index).unwrap().index(),
                field.index()
            );
        }
        #[cfg(feature = "test-extra")]
        {
            let generic = TypeMetadata::of::<Generic<u64>>();
            assert_eq!(generic.fields().len(), 1);
            assert_eq!(generic.fields()[0].index(), 1);
            assert_eq!(
                TypeDescriptor::of::<Generic<u64>>()
                    .fields()
                    .get_at(0)
                    .unwrap()
                    .index(),
                1
            );
        }
        #[cfg(all(feature = "test-extra", feature = "enum-probe"))]
        {
            let metadata = TypeMetadata::of::<CfgEnum>().as_enum().unwrap();
            let descriptor = TypeDescriptor::of::<CfgEnum>();
            assert_eq!(metadata.variants().len(), 2);
            assert_eq!(metadata.variants()[0].rust_name(), "Data");
            assert_eq!(metadata.variants()[1].rust_name(), "Pair");
            let variant = descriptor.variant("Data").unwrap();
            assert_eq!(metadata.variants()[0].index(), variant.index());
            assert_eq!(metadata.variants()[0].index(), 1);
            assert_eq!(variant.index(), 1);
            assert_eq!(metadata.variants()[0].field("payload").unwrap().index(), 1);
            assert_eq!(variant.fields().get("payload").unwrap().index(), 1);
            assert_eq!(
                metadata.variants()[0].fields().len(),
                1 + usize::from(cfg!(feature = "field-enabled"))
            );
            let pair = descriptor.variant("Pair").unwrap();
            assert_eq!(metadata.variants()[1].index(), 2);
            assert_eq!(pair.index(), 2);
            assert_eq!(
                metadata.variants()[1].fields().len(),
                1 + usize::from(cfg!(feature = "field-enabled"))
            );
            assert_eq!(metadata.variants()[1].fields().last().unwrap().index(), 1);
            assert_eq!(pair.fields().iter().last().unwrap().index(), 1);
            #[cfg(feature = "via-cfg-attr")]
            let generic_enum = GenericCfgEnum::Data(5_u64, 6_u64);
            #[cfg(not(feature = "via-cfg-attr"))]
            let generic_enum = GenericCfgEnum::Data(5_u64);
            assert_eq!(generic_enum.clone(), generic_enum);
            assert_eq!(
                format!("{generic_enum}"),
                if cfg!(feature = "via-cfg-attr") {
                    "Data(5, 6)"
                } else {
                    "Data(5)"
                }
            );
            let generic_metadata = TypeMetadata::of::<GenericCfgEnum<u64>>().as_enum().unwrap();
            let generic_descriptor = TypeDescriptor::of::<GenericCfgEnum<u64>>();
            let data_metadata = generic_metadata
                .variants()
                .iter()
                .find(|variant| variant.rust_name() == "Data")
                .unwrap();
            let data_descriptor = generic_descriptor.variant("Data").unwrap();
            assert_eq!(
                data_metadata.fields().len(),
                1 + usize::from(cfg!(feature = "via-cfg-attr"))
            );
            for (position, field) in data_metadata.fields().iter().enumerate() {
                assert_eq!(field.index(), position + 1);
                assert_eq!(
                    data_descriptor
                        .fields()
                        .iter()
                        .nth(position)
                        .unwrap()
                        .index(),
                    position + 1
                );
            }
            assert_eq!(
                TypeMetadata::of::<GenericCfgEnum<u64>>().role(),
                model_runtime::metadata::ModelRole::Enum
            );
        }
        #[cfg(feature = "test-extra")]
        {
            #[cfg(not(feature = "bound-enabled"))]
            {
                let value = ConditionalBound::<NoTraitType>::Empty(PhantomData);
                let _ = value.clone();
                let _ = value == value;
                assert!(format!("{value}").starts_with("Empty(PhantomData<"));
            }
            #[cfg(feature = "bound-enabled")]
            {
                let value = ConditionalBound::Payload(5_u64);
                let _ = value.clone();
                assert_eq!(format!("{value}"), "Payload(5)");
            }

            #[cfg(not(feature = "zero-field"))]
            let payload = ZeroFieldPayload::<NoTraitType>::Payload();
            #[cfg(feature = "zero-field")]
            let payload = ZeroFieldPayload::<NoTraitType>::Payload(3_u64, 4_u32);
            let _ = payload.clone();
            assert_eq!(format!("{payload}"), if cfg!(feature = "zero-field") { "Payload(3, 4)" } else { "Payload" });

            let _ = ConditionalDefault::<NoTraitType>::default();
            assert_eq!(format!("{}", ConditionalTransparent(DisplayType)), "display");
        }
        #[cfg(feature = "test-extra")]
        {
            let tuple = TypeMetadata::of::<CfgTuple>();
            let descriptor = TypeDescriptor::of::<CfgTuple>();
            assert_eq!(tuple.fields().len(), 1);
            assert_eq!(tuple.fields()[0].index(), 0);
            assert_eq!(descriptor.fields().get_at(0).unwrap().index(), 0);
            let _ = CfgTuple(String::new());
            let tuple = GenericTuple::<u64>(5);
            assert_eq!(tuple.clone(), tuple);
            assert_eq!(format!("{tuple}"), "GenericTuple(5)");
            assert_eq!(GenericTuple::<u64>::default(), GenericTuple(0));
            assert_eq!(format!("{}", GenericTransparent::<u64>(5)), "5");
        }
    }
}
