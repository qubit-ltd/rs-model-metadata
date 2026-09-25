# `inline(always)` 审计

[English](inline-always-audit.md)

审计日期：2026-09-24。本清单覆盖 runtime 与 derive 源码基线中的 224 处生产提示，逐项记录
每个原有标注方法的处理。独立的
[覆盖映射 fixture](../tests/fixtures/coverage_mapping/src/record.rs) 中两处提示有意保留，
用于复现覆盖率映射问题。

逐方法测试没有证明强制内联能带来可重复收益。190 个精简 `const` 访问器/构造函数与 24 个
精简转发/切片访问器改用普通 `#[inline]`，将决定权留给优化器。其余十个涉及运行时查找、分支、
函数指针或缓存的函数移除了内联提示。管线基准测量可观察的属性查询、图、计划和执行路径；
这些结果不能归因到某个单独访问器。

调整提示后运行 `cargo +1.94.0 bench --locked --bench model_pipeline
--features validation -- --quick`，单字段热 registry 查询测得 39.3 ns，8 字段
53.8 ns，32 字段 48.8 ns。Criterion 相比紧邻基线只在单字段项报告小幅显著改善；
8/32 字段以及 graph、plan、execution 阶段均无显著变化。`property_output` 快速基准
也没有显著回归，除一个亚纳秒 native getter 项改善外，其余变化均不显著。快速基准
存在噪声，也无法隔离单个内联提示的影响。

| 源文件 | 保留为普通 `#[inline]` | 移除内联提示 |
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

当前生产代码已没有 `#[inline(always)]` 属性。只有在相同工具链和基准配置下取得可重复的
前后测量后，才重新评估普通内联提示。
