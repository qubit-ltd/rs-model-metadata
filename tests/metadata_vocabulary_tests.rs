// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Declaration facts agree with the behavior generated for consumers.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::metadata::DeclaredEntityTarget;
use qubit_model_metadata::metadata::DeclaredEntityTargetKind;
use qubit_model_metadata::metadata::DependencyBindingMetadata;
use qubit_model_metadata::metadata::FieldReferenceMetadata;
use qubit_model_metadata::metadata::FieldUniqueMetadata;
use qubit_model_metadata::metadata::IdentifierAssignment;
use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ObjectPath;
use qubit_model_metadata::metadata::OnNone;
use qubit_model_metadata::metadata::PropertyPath;
use qubit_model_metadata::metadata::RedactModeMetadata;
use qubit_model_metadata::metadata::RedactPosition;
use qubit_model_metadata::metadata::ReferenceSelection;
use qubit_model_metadata::metadata::Sensitivity;
use qubit_model_metadata::metadata::SerdeBehaviorSource;
use qubit_model_metadata::metadata::TargetMode;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::metadata::ValidatorMetadata;
use serde_json::from_value;
use serde_json::json;
use serde_json::to_value;

// Flatten belongs to the plain Serde path; redacted output rejects it.
#[Model(no_redact)]
struct WireRecord {
    #[serde(rename(serialize = "outgoing", deserialize = "incoming"))]
    name: String,
    #[serde(skip_serializing)]
    secret: String,
    #[serde(skip_deserializing)]
    computed: String,
    #[serde(flatten)]
    extra: BTreeMap<String, String>,
    tags: Vec<String>,
    #[keep_serializing]
    kept: Vec<String>,
    #[serde(default)]
    optional: Option<String>,
    #[serde(with = "wire_text")]
    converted: String,
}

#[Model]
struct RedactedRecord {
    title: String,
    #[redact(level = "secret")]
    token: String,
    #[element(redact(level = "secret"))]
    labels: Vec<String>,
}

#[Model]
struct WrappedUnique {
    // The extra pointer layer exercises reflected text-capability traversal.
    #[allow(clippy::box_collection)]
    #[unique]
    text: Option<Box<String>>,
    #[unique]
    number: Option<Box<u64>>,
}

#[Model]
struct ScopedName {
    tenant: String,
    #[unique(respect_to(tenant), ignore_case = false)]
    name: String,
}

#[Entity(id = "vocabulary.Tenant")]
struct Tenant {
    #[identifier(assigned_by = database)]
    id: Id,
}

#[Value]
struct Coordinate {
    #[key_part(order = 1)]
    northing: i64,
    #[key_part(order = 0)]
    easting: i64,
}

/// Key-part positions are explicit facts, independent of source field order.
#[test]
fn test_composite_key_positions_preserve_declared_order() {
    let metadata = TypeMetadata::of::<Coordinate>();
    let parts = metadata
        .fields()
        .iter()
        .map(|field| {
            (
                field.name().expect("named component"),
                field.key_part().expect("key component").order(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(parts, [("northing", 1), ("easting", 0)]);
}

static PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Makes provider invocation observable without using the global registry.
fn tenant_metadata() -> &'static TypeMetadata {
    PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);
    TypeMetadata::of::<Tenant>()
}

/// Inspecting target provenance must not invoke a lazy metadata provider.
#[test]
fn test_reference_target_inspection_is_lazy_and_preserves_assignment() {
    static TARGET: DeclaredEntityTarget = DeclaredEntityTarget::RustType(tenant_metadata);
    static SELECTION: ReferenceSelection = ReferenceSelection::Property(PropertyPath::new(&["id"]));
    static CURRENT: ObjectPath = ObjectPath::current();
    let reference = FieldReferenceMetadata::new(&TARGET, &SELECTION, true, Some(&CURRENT));
    assert_eq!(reference.target().kind(), DeclaredEntityTargetKind::RustType);
    assert_eq!(reference.target().model_id(), None);
    assert!(reference.existing());
    assert_eq!(reference.selection(), &SELECTION);
    assert!(reference.path().expect("explicit current object").steps().is_empty());
    assert!(format!("{reference:?}").contains("RustType"));
    assert_eq!(PROVIDER_CALLS.load(Ordering::SeqCst), 0);
    let target = reference.target().metadata().expect("invoke provider explicitly");
    assert_eq!(PROVIDER_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(target.model_id(), Some(ModelId::new("vocabulary.Tenant")));
    let identifier = target
        .field("id")
        .expect("identifier field")
        .identifier()
        .expect("identifier policy");
    assert_eq!(identifier.assigned_by(), IdentifierAssignment::Database);

    let by_id = DeclaredEntityTarget::ModelId(ModelId::new("vocabulary.Tenant"));
    assert_eq!(by_id.kind(), DeclaredEntityTargetKind::ModelId);
    assert_eq!(by_id.model_id(), target.model_id());
    assert!(
        by_id.metadata().is_none(),
        "an ID does not implicitly resolve a registry"
    );
    assert_eq!(PROVIDER_CALLS.load(Ordering::SeqCst), 1);
}

/// Deferred declarations preserve explicit overrides without inventing
/// defaults.
#[test]
fn test_unique_definition_preserves_explicit_and_deferred_policies() {
    static SCOPE: [PropertyPath<'static>; 1] = [PropertyPath::new(&["tenant"])];
    let deferred = FieldUniqueMetadata::for_definition(&SCOPE, None);
    assert_eq!(deferred.effective_ignore_case(), None);
    assert_eq!(deferred.declared_ignore_case(), None);
    for explicit in [false, true] {
        let definition = FieldUniqueMetadata::for_definition(&SCOPE, Some(explicit));
        let concrete = FieldUniqueMetadata::new(&SCOPE, explicit);
        assert_eq!(definition.effective_ignore_case(), concrete.effective_ignore_case());
        assert_eq!(definition.declared_ignore_case(), Some(explicit));
        assert!(definition.is_scoped());
        assert_eq!(definition.respect_to(), concrete.respect_to());
    }
}

/// A dependency constructor rejects an unnamed slot before it reaches binding.
#[test]
#[should_panic(expected = "validator dependency name cannot be empty")]
fn test_dependency_rejects_empty_name() {
    let _ = DependencyBindingMetadata::new("", PropertyPath::new(&["id"]));
}

/// An empty dependency path cannot describe a field read.
#[test]
#[should_panic(expected = "validator dependency path cannot be empty")]
fn test_dependency_rejects_empty_path() {
    let _ = DependencyBindingMetadata::new("owner", PropertyPath::new(&[]));
}

/// Raw occurrence construction still requires a nonempty source identifier.
#[test]
#[should_panic(expected = "validator ID cannot be empty")]
fn test_validator_rejects_empty_identifier() {
    let _ = ValidatorMetadata::new("", &[], &[]);
}

/// Two dependency sources cannot claim the same validator signature slot.
#[test]
#[should_panic(expected = "validator dependency names must be unique")]
fn test_validator_rejects_duplicate_dependency_slots() {
    static BINDINGS: [DependencyBindingMetadata; 2] = [
        DependencyBindingMetadata::new("owner", PropertyPath::new(&["first"])),
        DependencyBindingMetadata::new("owner", PropertyPath::new(&["second"])),
    ];
    let _ = ValidatorMetadata::new_bound("example.rule", &[], &[], &BINDINGS, TargetMode::Value, OnNone::Skip);
}

/// Container input cannot simultaneously request absent-value expansion policy.
#[test]
#[should_panic(expected = "container validators cannot reject missing expanded values")]
fn test_validator_rejects_incompatible_container_policy() {
    let _ = ValidatorMetadata::new_bound("example.rule", &[], &[], &[], TargetMode::Container, OnNone::Reject);
}

/// Field and selector redaction facts explain the generated protected output.
#[test]
fn test_redaction_metadata_matches_output_positions() {
    let value = RedactedRecord {
        title: "visible title".to_owned(),
        token: "private token".to_owned(),
        labels: vec!["private label".to_owned()],
    };
    let wire = to_value(&value).expect("redacted serialization");
    assert_eq!(wire["title"], "visible title");
    assert!(!wire.to_string().contains("private token"));
    assert!(!wire.to_string().contains("private label"));
    let debug = format!("{value:?}");
    assert!(!debug.contains("private token"));
    assert!(!debug.contains("private label"));
    let metadata = TypeMetadata::of::<RedactedRecord>();
    let token = metadata
        .field("token")
        .expect("token field")
        .redact()
        .expect("field policy");
    assert_eq!(token.sensitivity(), Some(Sensitivity::Secret));
    assert_eq!(token.mode(), RedactModeMetadata::Level);
    assert_eq!(token.position(), RedactPosition::Field);
    let sequence = metadata
        .field("labels")
        .expect("labels field")
        .sequence_constraint()
        .expect("sequence metadata");
    let labels = sequence.element().expect("element policy");
    let redaction = labels.redact().expect("element redaction");
    assert_eq!(redaction.sensitivity(), Some(Sensitivity::Secret));
    assert_eq!(redaction.mode(), RedactModeMetadata::Level);
    assert_eq!(redaction.position(), RedactPosition::Element);
}

/// Effective uniqueness follows concrete capabilities through nested wrappers.
#[test]
fn test_unique_policy_distinguishes_wrapped_text_from_explicit_scope() {
    let metadata = TypeMetadata::of::<WrappedUnique>();
    for (field, expected) in [("text", true), ("number", false)] {
        let policy = metadata
            .field(field)
            .expect("wrapped field")
            .unique()
            .expect("unique policy");
        assert_eq!(policy.declared_ignore_case(), None);
        assert_eq!(policy.effective_ignore_case(), Some(expected));
        assert!(!policy.is_scoped());
    }
    let scoped = TypeMetadata::of::<ScopedName>()
        .field("name")
        .expect("scoped field")
        .unique()
        .expect("unique policy");
    assert!(scoped.is_scoped());
    assert_eq!(scoped.respect_to()[0].segments(), &["tenant"]);
    assert_eq!(scoped.declared_ignore_case(), Some(false));
    assert_eq!(scoped.effective_ignore_case(), Some(false));
}

/// A real Serde adapter used to distinguish declared conversion from renaming.
mod wire_text {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    /// Writes the uppercase wire representation.
    pub fn serialize<S: Serializer>(value: &str, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_uppercase())
    }

    /// Restores the lowercase application representation.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
        String::deserialize(deserializer).map(|value| value.to_lowercase())
    }
}

/// Effective field metadata explains both wire directions and omission origins.
#[test]
fn test_serde_metadata_matches_generated_wire_behavior() {
    let value = WireRecord {
        name: "note".to_owned(),
        secret: "hidden".to_owned(),
        computed: "output".to_owned(),
        extra: BTreeMap::from([("extension".to_owned(), "value".to_owned())]),
        tags: Vec::new(),
        kept: Vec::new(),
        optional: None,
        converted: "mixed".to_owned(),
    };
    assert_eq!(
        to_value(&value).expect("serialize model"),
        json!({
            "outgoing": "note", "computed": "output", "extension": "value",
            "kept": [], "converted": "MIXED"
        })
    );
    let decoded: WireRecord = from_value(json!({
        "incoming": "note", "secret": "input", "extension": "value", "converted": "MIXED"
    }))
    .expect("deserialize model defaults");
    assert_eq!(decoded.name, "note");
    assert_eq!(decoded.secret, "input");
    assert!(decoded.computed.is_empty());
    assert!(decoded.tags.is_empty());
    assert!(decoded.kept.is_empty());
    assert!(decoded.optional.is_none());
    assert_eq!(decoded.extra.get("extension").map(String::as_str), Some("value"));
    assert_eq!(decoded.converted, "mixed");

    let metadata = TypeMetadata::of::<WireRecord>();
    let name = metadata.field("name").expect("name field").serde();
    assert_eq!(name.serialize_name(), Some("outgoing"));
    assert_eq!(name.deserialize_name(), Some("incoming"));
    assert!(!name.skip_serializing());
    assert!(!name.skip_deserializing());
    assert!(
        metadata
            .field("secret")
            .expect("secret field")
            .serde()
            .skip_serializing()
    );
    assert!(
        metadata
            .field("computed")
            .expect("computed field")
            .serde()
            .skip_deserializing()
    );
    assert!(metadata.field("extra").expect("extra field").serde().flatten());
    assert_eq!(
        metadata.field("converted").expect("converted field").serde().with(),
        Some("wire_text")
    );
    let tags = metadata.field("tags").expect("tags field").serde();
    assert!(tags.default());
    assert_eq!(tags.default_source(), SerdeBehaviorSource::ModelDefault);
    assert_eq!(tags.omit_source(), SerdeBehaviorSource::ModelDefault);
    assert_eq!(
        metadata.field("kept").expect("kept field").serde().omit_source(),
        SerdeBehaviorSource::Suppressed
    );
    assert_eq!(
        metadata
            .field("optional")
            .expect("optional field")
            .serde()
            .default_source(),
        SerdeBehaviorSource::Explicit
    );
}
