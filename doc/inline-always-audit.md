# `inline(always)` audit

[简体中文](inline-always-audit.zh_CN.md)

Review date: 2026-09-24. This inventory covers the 224 production hints in the
runtime and derive source at the audit baseline. It records the disposition of
each annotated method; the two hints in
[`tests/fixtures/coverage_mapping`](../tests/fixtures/coverage_mapping/src/record.rs)
remain intentionally as a coverage-mapping reproducer.

No per-method benchmark demonstrated a repeatable gain from forcing inlining.
Thin `const` accessors and constructors (190 methods) and thin forwarding or
slice accessors (24 methods) now use ordinary `#[inline]`, which leaves the
optimizer free to choose. Ten methods with runtime lookup, branching, or
function-pointer/cache behavior leave the decision to the compiler without an
inline hint. The pipeline benchmarks measure the observable property lookup,
graph, plan, and execution paths; they are not evidence for an individual
accessor.

With the hints adjusted, `cargo +1.94.0 bench --locked --bench
model_pipeline --features validation -- --quick` measured 39.3 ns for one-field
warm registry lookup, 53.8 ns for eight fields, and 48.8 ns for 32 fields.
Criterion reported a small significant improvement for the one-field case
against its immediately preceding baseline; the eight- and 32-field cases and
graph, plan, and execution stages showed no significant change. The
`property_output` quick run also showed no significant regression; its reported
changes were nonsignificant except one sub-nanosecond native getter improvement.
Quick runs are noisy and do not isolate individual inline hints.

| Source file | Ordinary `#[inline]` | Hint removed |
| --- | --- | --- |
| `derive/src/ir/located.rs` | `value`, `span` | — |
| `src/abi_violation.rs` | `code`, `message` | — |
| `src/constraint/decimal_constraint.rs` | `precision`, `scale`, `rounding`, `semantic`, `min`, `max`, `min_inclusive`, `max_inclusive` | — |
| `src/constraint/map_constraint.rs` | `min_entries`, `max_entries`, `key`, `value` | — |
| `src/constraint/sequence_constraint.rs` | `min_items`, `max_items`, `unique_items`, `element` | — |
| `src/constraint/temporal_constraint.rs` | `new`, `precision` | — |
| `src/constraint/text_constraint.rs` | `min_chars`, `max_chars`, `min_bytes`, `max_bytes`, `allowed_chars`, `is_non_blank`, `format` | — |
| `src/field_metadata.rs` | `location`, `declaration`, `reflect`, `definition`, `index`, `name`, `visibility`, `attributes`, `constraints`, `validators`, `serde` | `type_ref`, `descriptor` |
| `src/generic/generic_model_metadata.rs` | `role`, `definition`, `fields`, `variants` | — |
| `src/local_property_set.rs` | `properties` | — |
| `src/metadata_vocabulary.rs` | `name`, `value`, `new` (7), `assigned_by`, `declared_ignore_case`, `effective_ignore_case`, `respect_to`, `ignore_case`, `is_scoped`, `order`, `kind`, `model_id`, `target` (2), `selection`, `existing`, `path` (2), `object_path`, `property`, `declaration`, `name`, `declared_id`, `params`, `depends_on`, `dependency_bindings`, `on_none`, `codec` (2), `source`, `sensitivity`, `mode`, `position` (2), `serialize_name`, `deserialize_name`, `skip_serializing`, `skip_deserializing`, `flatten`, `with`, `default`, `default_source`, `omit_source`, `constraints`, `validators`, `redact` | `type_id`, `type_name` |
| `src/model_id.rs` | `new`, `as_str` | — |
| `src/model_id/model_id_buf.rs` | `as_str` | — |
| `src/model_impl_metadata.rs` | `fragments`, `try_properties` | — |
| `src/private/reflect_codegen.rs` | — | `reflected_type_ref` |
| `src/property.rs` | `len`, `is_empty`, `get`, `error`, `rust_method_name` (2), `output_type`, `output_kind`, `input_type`, `name`, `type_ref`, `descriptor`, `field`, `getter`, `setter` | — |
| `src/property_build_error.rs` | `kind`, `property_name` | — |
| `src/property_build_errors.rs` | `errors` | — |
| `src/property_fragment.rs` | `name`, `type_ref`, `source` | — |
| `src/registry/error.rs` | `abi_cause`, `kind`, `model_id`, `capability_id`, `expected_adapter_type`, `actual_adapter_type`, `sources` | — |
| `src/registry/model_entry.rs` | `model_id`, `source`, `metadata`, `generic_metadata` | — |
| `src/registry/model_registry.rs` | `entries`, `generic_definitions` | `metadata`, `generic`, `by_type_id`, `source`, `concrete_entries` |
| `src/relation/declaration_location.rs` | `unknown`, `with_selector` | — |
| `src/relation/field_location.rs` | `owner`, `variant`, `index` | — |
| `src/relation/property_path.rs` | `segments`, `is_empty` | — |
| `src/resolve/error.rs` | `errors` | — |
| `src/resolve/graph.rs` | `declaration` (2), `context_requirement`, `target` (2), `property` (2), `source`, `projection`, `projector`, `dependencies`, `models`, `projection_producers`, `registry`, `declarations`, `field` | — |
| `src/role.rs` | `identifier` (2), `source`, `is_open`, `is_fixed`, `is_transparent`, `transparent_field`, `canonical_codec`, `role` | — |
| `src/type_metadata.rs` | `descriptor`, `model_id`, `fields`, `generic_definition`, `concrete_generic`, `role`, `role_metadata`, `as_entity`, `as_projection`, `as_model`, `as_enum`, `as_value` | — |
| `src/type_metadata/enum_metadata.rs` | `variants` | — |
| `src/type_metadata/enum_variant_metadata.rs` | `reflect`, `definition`, `index`, `rust_name`, `canonical_name`, `serialized_name`, `deserialized_name`, `fields`, `field_at`, `is_default` | — |
| `src/validation/model_rule_binding.rs` | `rule_id`, `input_type`, `validator` | — |
| `src/validation/validation_build_errors.rs` | `len`, `is_empty`, `as_slice`, `iter`, `index`, `into_iter`, `as_ref` | — |
| `src/validation/validation_options.rs` | `segments`, `mode`, `selection`, `max_depth`, `max_nodes`, `max_violations`, `max_comparisons` | — |
| `src/validation/validation_plan.rs` | `binding_count`, `root`, `graph`, `model_rules`, `bindings` | — |

The current source has no production `#[inline(always)]` attributes. Reconsider
ordinary inline hints only with repeatable before/after measurements under the
same toolchain and benchmark configuration.
