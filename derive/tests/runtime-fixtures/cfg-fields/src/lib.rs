use model_runtime::__private::TypeDescriptor;
use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::{Enum, Model, Value};

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
    Data(T),
}

#[Value]
#[cfg(feature = "test-extra")]
struct CfgTuple(String, #[cfg(any())] MissingType);

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
            let _ = GenericCfgEnum::Data(5_u64);
            assert_eq!(
                TypeMetadata::of::<GenericCfgEnum<u64>>().role(),
                model_runtime::metadata::ModelRole::Enum
            );
        }
        #[cfg(feature = "test-extra")]
        {
            let tuple = TypeMetadata::of::<CfgTuple>();
            let descriptor = TypeDescriptor::of::<CfgTuple>();
            assert_eq!(tuple.fields().len(), 1);
            assert_eq!(tuple.fields()[0].index(), 0);
            assert_eq!(descriptor.fields().get_at(0).unwrap().index(), 0);
            let _ = CfgTuple(String::new());
        }
    }
}
