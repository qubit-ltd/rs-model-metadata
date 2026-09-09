//! Default model capabilities and actual output contracts.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_reflect::Reflect;

static POLICY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[Model]
struct Label {
    text: String,
}

/// Role macros supply usable Rust capabilities without handwritten derives.
#[test]
fn test_default_model_capabilities() {
    /// Checks the observable default trait contract at compilation.
    fn traits<
        T: Clone
            + Eq
            + std::hash::Hash
            + std::fmt::Debug
            + std::fmt::Display
            + serde::Serialize
            + for<'de> serde::Deserialize<'de>,
    >() {
    }
    traits::<Label>();
    let label = Label {
        text: "visible".to_owned(),
    };
    assert_eq!(label.clone(), label);
    assert!(format!("{label}").contains("visible"));
}

#[Model(no_eq, no_display, no_debug, no_serialize, no_deserialize, no_redact)]
struct Measurement {
    value: f64,
}

/// Disabling Eq removes the coupled Hash implementation, permitting floats.
#[test]
fn test_eq_opt_out_allows_non_eq_fields() {
    let value = Measurement { value: 0.5 };
    assert!(value == value.clone());
}

#[Model]
struct Defaults {
    optional: Option<String>,
    values: Vec<String>,
    #[keep_serializing]
    retained: Vec<String>,
    #[serde(skip_serializing)]
    #[keep_serializing]
    hidden: Vec<String>,
}

/// Missing containers default while explicit Serde skips keep precedence.
#[test]
fn test_named_container_defaults() {
    let value: Defaults = serde_json::from_str("{}").expect("container defaults");
    assert_eq!(
        serde_json::to_value(value).expect("serialize"),
        serde_json::json!({"retained": []})
    );
}

#[Model]
struct Secrets {
    #[redact(level = "secret")]
    token: String,
    #[redact(skip)]
    hidden: Vec<String>,
}

/// Every generated output uses the delegated redaction policy.
#[test]
fn test_default_outputs_protect_secrets() {
    let _guard = POLICY_LOCK.lock().expect("policy lock");
    let value = Secrets {
        token: "raw-secret".into(),
        hidden: vec!["hidden-secret".into()],
    };
    let encoded = serde_json::to_value(&value).expect("serialize");
    assert!(encoded.get("hidden").is_none());
    for text in [format!("{value:?}"), format!("{value}"), encoded.to_string()] {
        assert!(!text.contains("raw-secret"));
        assert!(!text.contains("hidden-secret"));
    }
}

#[derive(Reflect)]
struct NoValueTraits;

#[Model(no_redact, no_debug, no_display, no_serialize, no_deserialize)]
struct Marker<T> {
    marker: std::marker::PhantomData<T>,
}

/// Generic role capabilities constrain field shapes rather than every
/// parameter.
#[test]
fn test_generic_capabilities_avoid_unnecessary_bounds() {
    let value = Marker::<NoValueTraits> {
        marker: std::marker::PhantomData,
    };
    assert!(value == value.clone());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&value, &mut hasher);
}

#[Value(transparent)]
struct HiddenValue(#[redact(skip)] String);

#[Enum]
enum HiddenPayload {
    Tuple(#[redact(skip)] Vec<String>, u32),
    Named {
        #[redact(skip)]
        value: Option<String>,
        count: u32,
    },
}

/// Skip omits complete payloads, while disabled policy restores their values.
#[test]
fn test_redact_skip_shapes_and_disabled_restoration() {
    use model_runtime::__private::redact::RedactionPolicy;
    use model_runtime::__private::redact::Redactor;
    let _guard = POLICY_LOCK.lock().expect("policy lock");
    let value = HiddenValue("hidden-value".into());
    let tuple = HiddenPayload::Tuple(vec!["hidden-tuple".into()], 2);
    let named = HiddenPayload::Named {
        value: Some("hidden-named".into()),
        count: 3,
    };
    assert_eq!(
        serde_json::to_value(&value).expect("transparent skip"),
        serde_json::Value::Null
    );
    for text in [
        format!("{value:?}"),
        format!("{value}"),
        serde_json::to_string(&tuple).expect("tuple"),
        serde_json::to_string(&named).expect("named"),
    ] {
        assert!(!text.contains("hidden-"));
    }
    let previous = Redactor::replace_application_default(Redactor::new(RedactionPolicy::disabled()));
    let restored = (
        serde_json::to_string(&value),
        serde_json::to_string(&tuple),
        serde_json::to_string(&named),
    );
    let _ = Redactor::replace_application_default(previous);
    assert!(restored.0.expect("value restored").contains("hidden-value"));
    assert!(restored.1.expect("tuple restored").contains("hidden-tuple"));
    assert!(restored.2.expect("named restored").contains("hidden-named"));
}

#[Model(no_redact)]
struct NestedOutput {
    nested: Secrets,
}

/// Opting out of outer redaction preserves the nested type's own safe output.
#[test]
fn test_no_redact_preserves_nested_safe_output() {
    let _guard = POLICY_LOCK.lock().expect("policy lock");
    let value = NestedOutput {
        nested: Secrets {
            token: "raw-secret".into(),
            hidden: vec!["hidden-secret".into()],
        },
    };
    for text in [
        format!("{value:?}"),
        format!("{value}"),
        serde_json::to_string(&value).expect("nested output"),
    ] {
        assert!(!text.contains("raw-secret"));
        assert!(!text.contains("hidden-secret"));
    }
}

#[Model]
struct SelectorSecrets {
    #[element(redact(level = "secret"))]
    sequence: Vec<String>,
    #[map_value(redact(level = "secret"))]
    values: std::collections::BTreeMap<String, String>,
    #[map_key(redact(level = "secret"))]
    #[map_value(redact(level = "secret"))]
    keys: std::collections::BTreeMap<String, String>,
}

/// Selector output delegates level handling and collision detection to
/// rs-redact.
#[test]
fn test_selector_redaction_and_map_key_collisions() {
    let _guard = POLICY_LOCK.lock().expect("policy lock");
    let mut value = SelectorSecrets {
        sequence: vec!["raw-sequence".into()],
        values: [("visible-key".into(), "raw-value".into())].into(),
        keys: [("raw-key".into(), "raw-key-value".into())].into(),
    };
    for text in [
        serde_json::to_string(&value).expect("selector output"),
        format!("{value:?}"),
        format!("{value}"),
    ] {
        assert!(!text.contains("raw-"), "unredacted selector: {text}");
        assert!(text.contains("visible-key"));
    }
    value.keys.insert("another-key".into(), "another-value".into());
    assert!(
        serde_json::to_string(&value).is_err(),
        "masked key collisions must not lose entries"
    );
}

#[Model(no_redact, no_serialize, no_deserialize, default, ord)]
struct OrderedMarker<T> {
    marker: std::marker::PhantomData<T>,
}

#[Enum(no_redact, no_debug)]
enum PlainDisplay {
    Named { value: u32 },
    Tuple(String),
    Unit,
}

/// Default, ordering and formatting use the capabilities of stored fields.
#[test]
fn test_independent_precise_optional_capabilities() {
    let first = OrderedMarker::<NoValueTraits>::default();
    let second = first.clone();
    assert_eq!(first.cmp(&second), core::cmp::Ordering::Equal);
    assert!(format!("{first:?}").contains("marker"));
    assert!(format!("{}", PlainDisplay::Named { value: 7 }).contains('7'));
    assert!(format!("{}", PlainDisplay::Tuple("text".into())).contains("text"));
    assert!(format!("{}", PlainDisplay::Unit).contains("Unit"));
}

#[Enum(no_redact, no_serialize, no_deserialize, ord)]
#[repr(i16)]
enum OrderedPayload<T> {
    Later(std::marker::PhantomData<T>) = 9,
    Earlier(std::marker::PhantomData<T>) = -2,
}

/// Generic Enum ordering honors discriminants without adding unused T bounds.
#[test]
fn test_generic_enum_ordering_uses_discriminants() {
    let earlier = OrderedPayload::<NoValueTraits>::Earlier(std::marker::PhantomData);
    let later = OrderedPayload::<NoValueTraits>::Later(std::marker::PhantomData);
    assert!(earlier < later);
}

#[Model(no_redact, no_serialize, no_deserialize)]
struct NoSerde {
    values: Vec<String>,
}

/// Output opt-outs do not leave generated Serde helper attributes behind.
#[test]
fn test_complete_serde_opt_out() {
    assert!(NoSerde { values: vec![] }.values.is_empty());
}

#[Model(no_redact, no_display, no_serialize, no_deserialize)]
struct Recursive<T> {
    value: T,
    next: Option<Box<Recursive<T>>>,
}

/// Recursive storage does not create a circular generated trait obligation.
#[test]
fn test_recursive_generic_structural_capabilities() {
    let value = Recursive {
        value: 1u32,
        next: Some(Box::new(Recursive { value: 2, next: None })),
    };
    assert_eq!(value, value.clone());
}

#[Model]
struct RecursiveOutput<T> {
    value: T,
    next: Option<Box<RecursiveOutput<T>>>,
}

/// Default output and deserialization remain usable for finite recursive
/// values.
#[test]
fn test_recursive_generic_default_output() {
    let value = RecursiveOutput {
        value: 1u32,
        next: None,
    };
    let json = serde_json::to_string(&value).expect("recursive output");
    let decoded: RecursiveOutput<u32> = serde_json::from_str(&json).expect("recursive input");
    assert_eq!(value, decoded);
}
