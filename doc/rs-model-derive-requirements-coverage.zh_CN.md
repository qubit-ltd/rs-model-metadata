# 需求覆盖台账

本文把需求规范中的每个稳定需求 ID 映射到主要验证面。表中的“自动化”表示该组规则由所列测试覆盖；它不表示单个测试逐字复述需求。跨 crate 消费、文档完整性和领域语义同时依赖审查与集成验证。

| 需求组 | 需求 ID | 覆盖方式 | 主要证据 |
| --- | --- | --- | --- |
| ACC | `REQ-ACC-001`、`REQ-ACC-002`、`REQ-ACC-003`、`REQ-ACC-004`、`REQ-ACC-005`、`REQ-ACC-006`、`REQ-ACC-007`、`REQ-ACC-008` | 自动化 | `tests/property_metadata_tests.rs`、`rs-model-metadata/tests/property_tests.rs` |
| CAP | `REQ-CAP-001`、`REQ-CAP-002`、`REQ-CAP-003`、`REQ-CAP-004`、`REQ-CAP-005`、`REQ-CAP-006`、`REQ-CAP-007`、`REQ-CAP-008`、`REQ-CAP-009`、`REQ-CAP-010` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/ui_roles/fail/{no_copy,unsafe_existing_derives,redact_conflict}.rs` |
| CODEC | `REQ-CODEC-001`、`REQ-CODEC-002`、`REQ-CODEC-003`、`REQ-CODEC-004`、`REQ-CODEC-005`、`REQ-CODEC-006`、`REQ-CODEC-007`、`REQ-CODEC-008` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/ui_roles/fail/codec_bound.rs` |
| CONS | `REQ-CONS-001`、`REQ-CONS-002`、`REQ-CONS-003`、`REQ-CONS-004`、`REQ-CONS-005`、`REQ-CONS-006`、`REQ-CONS-007` | 自动化 + 审查 | `cargo doc`、README/用户指南审查、`rs-platform` workspace check |
| DEC | `REQ-DEC-001`、`REQ-DEC-002`、`REQ-DEC-003`、`REQ-DEC-004`、`REQ-DEC-005`、`REQ-DEC-006`、`REQ-DEC-007`、`REQ-DEC-008`、`REQ-DEC-009`、`REQ-DEC-010` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/ui_roles/fail/{decimal_bounds,constraint_validation,constraint_type_targets}.rs` |
| ENT | `REQ-ENT-001`、`REQ-ENT-002`、`REQ-ENT-003`、`REQ-ENT-004`、`REQ-ENT-005`、`REQ-ENT-006`、`REQ-ENT-007` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/resolver_graph_constraints_tests.rs` |
| ENUM | `REQ-ENUM-001`、`REQ-ENUM-002`、`REQ-ENUM-003`、`REQ-ENUM-004`、`REQ-ENUM-005`、`REQ-ENUM-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/role_pipeline_tests.rs` |
| ERR | `REQ-ERR-001`、`REQ-ERR-002`、`REQ-ERR-003`、`REQ-ERR-004`、`REQ-ERR-005`、`REQ-ERR-006`、`REQ-ERR-010`、`REQ-ERR-011`、`REQ-ERR-012`、`REQ-ERR-013` | 自动化 | `tests/role_trybuild_tests.rs`（含 overflow、伪 Vec、精确 span） |
| FLD | `REQ-FLD-001`、`REQ-FLD-002`、`REQ-FLD-003`、`REQ-FLD-004`、`REQ-FLD-005`、`REQ-FLD-006`、`REQ-FLD-007`、`REQ-FLD-008` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`rs-model-metadata/tests/field_metadata_tests.rs` |
| GEN | `REQ-GEN-001`、`REQ-GEN-002`、`REQ-GEN-003`、`REQ-GEN-004`、`REQ-GEN-005`、`REQ-GEN-006`、`REQ-GEN-007`、`REQ-GEN-008` | 自动化 | `tests/role_pipeline_tests.rs`、`tests/role_metadata_runtime_tests.rs`、`rs-model-metadata/tests/metadata_registry_tests.rs` |
| ID | `REQ-ID-001`、`REQ-ID-002`、`REQ-ID-003`、`REQ-ID-004`、`REQ-ID-005`、`REQ-ID-006`、`REQ-ID-007` | 自动化 | `tests/ui_roles/fail/identifier_type.rs`、`tests/role_metadata_runtime_tests.rs` |
| KEY | `REQ-KEY-001`、`REQ-KEY-002`、`REQ-KEY-003`、`REQ-KEY-004`、`REQ-KEY-005`、`REQ-KEY-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`rs-model-metadata/tests/metadata_resolver_tests.rs` |
| MAP | `REQ-MAP-001`、`REQ-MAP-002`、`REQ-MAP-003`、`REQ-MAP-004`、`REQ-MAP-005`、`REQ-MAP-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`rs-model-metadata/tests/constraint/map_constraint_tests.rs` |
| MDL | `REQ-MDL-001`、`REQ-MDL-002`、`REQ-MDL-003`、`REQ-MDL-004`、`REQ-MDL-005`、`REQ-MDL-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/role_pipeline_tests.rs` |
| META | `REQ-META-001`、`REQ-META-002`、`REQ-META-003`、`REQ-META-010`、`REQ-META-011`、`REQ-META-012`、`REQ-META-013`、`REQ-META-020`、`REQ-META-021`、`REQ-META-022`、`REQ-META-023`、`REQ-META-024`、`REQ-META-025`、`REQ-META-030`、`REQ-META-031`、`REQ-META-032`、`REQ-META-033`、`REQ-META-034`、`REQ-META-035`、`REQ-META-036`、`REQ-META-037`、`REQ-META-038`、`REQ-META-040`、`REQ-META-041`、`REQ-META-042`、`REQ-META-043`、`REQ-META-050`、`REQ-META-051`、`REQ-META-052`、`REQ-META-053`、`REQ-META-054`、`REQ-META-055`、`REQ-META-060`、`REQ-META-061`、`REQ-META-062`、`REQ-META-063`、`REQ-META-064`、`REQ-META-065`、`REQ-META-070`、`REQ-META-071`、`REQ-META-080`、`REQ-META-081`、`REQ-META-082`、`REQ-META-083`、`REQ-META-084`、`REQ-META-085`、`REQ-META-086`、`REQ-META-087` | 自动化 | `rs-model-metadata/tests/{abi_v3,type_metadata,reflect_facade,role_metadata,property}.rs`、derive runtime tests |
| OPAQUE | `REQ-OPAQUE-001`、`REQ-OPAQUE-002`、`REQ-OPAQUE-003`、`REQ-OPAQUE-004`、`REQ-OPAQUE-005`、`REQ-OPAQUE-006` | 自动化 | `tests/resolver_graph_constraints_tests.rs`、`rs-model-metadata/tests/metadata_resolver_tests.rs` |
| OUT | `REQ-OUT-001`、`REQ-OUT-002`、`REQ-OUT-003`、`REQ-OUT-004`、`REQ-OUT-005`、`REQ-OUT-006`、`REQ-OUT-007`、`REQ-OUT-008`、`REQ-OUT-009`、`REQ-OUT-010` | 自动化 | `tests/role_metadata_runtime_tests.rs`、trybuild capability conflicts |
| PRJ | `REQ-PRJ-001`、`REQ-PRJ-002`、`REQ-PRJ-003`、`REQ-PRJ-004`、`REQ-PRJ-005`、`REQ-PRJ-006`、`REQ-PRJ-007`、`REQ-PRJ-008` | 自动化 | `tests/projection_producer_tests.rs`、`tests/resolver_graph_constraints_tests.rs` |
| PROP | `REQ-PROP-001`、`REQ-PROP-002`、`REQ-PROP-003`、`REQ-PROP-004`、`REQ-PROP-005`、`REQ-PROP-006`、`REQ-PROP-007`、`REQ-PROP-008`、`REQ-PROP-009`、`REQ-PROP-010`、`REQ-PROP-011`、`REQ-PROP-012` | 自动化 | `tests/property_metadata_tests.rs`、`rs-model-metadata/tests/property_tests.rs` |
| QRY | `REQ-QRY-001`、`REQ-QRY-002`、`REQ-QRY-003`、`REQ-QRY-004`、`REQ-QRY-005`、`REQ-QRY-006`、`REQ-QRY-007`、`REQ-QRY-008`、`REQ-QRY-009`、`REQ-QRY-010`、`REQ-QRY-011`、`REQ-QRY-012`、`REQ-QRY-013`、`REQ-QRY-014`、`REQ-QRY-015` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`rs-model-metadata/tests/metadata_resolver_tests.rs` |
| RED | `REQ-RED-001`、`REQ-RED-002`、`REQ-RED-003`、`REQ-RED-004`、`REQ-RED-005`、`REQ-RED-006`、`REQ-RED-007`、`REQ-RED-008`、`REQ-RED-009`、`REQ-RED-010` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/ui_roles/fail/redact_conflict.rs` |
| REF | `REQ-REF-001`、`REQ-REF-002`、`REQ-REF-003`、`REQ-REF-004`、`REQ-REF-005`、`REQ-REF-006`、`REQ-REF-007`、`REQ-REF-008`、`REQ-REF-009` | 自动化 | `tests/resolver_graph_constraints_tests.rs`、linked workspace fixture |
| REG | `REQ-REG-001`、`REQ-REG-002`、`REQ-REG-003`、`REQ-REG-004`、`REQ-REG-005`、`REQ-REG-006`、`REQ-REG-010`、`REQ-REG-011`、`REQ-REG-012`、`REQ-REG-013`、`REQ-REG-014`、`REQ-REG-015`、`REQ-REG-016` | 自动化 | `tests/runtime_fixtures_tests.rs`、linked workspace fixture、`rs-model-metadata/tests/metadata_registry_tests.rs` |
| RES | `REQ-RES-001`、`REQ-RES-002`、`REQ-RES-003`、`REQ-RES-004`、`REQ-RES-005`、`REQ-RES-006`、`REQ-RES-007`、`REQ-RES-008` | 自动化 | `tests/resolver_graph_constraints_tests.rs`、`tests/projection_producer_tests.rs`、metadata resolver tests |
| ROLE | `REQ-ROLE-001`、`REQ-ROLE-002`、`REQ-ROLE-003`、`REQ-ROLE-004`、`REQ-ROLE-005`、`REQ-ROLE-006`、`REQ-ROLE-007`、`REQ-ROLE-008`、`REQ-ROLE-009` | 自动化 | `tests/role_pipeline_tests.rs`、`tests/ui_roles/fail/{invalid_shapes,role_field_rules,generic_entity}.rs` |
| SEL | `REQ-SEL-001`、`REQ-SEL-002`、`REQ-SEL-003`、`REQ-SEL-004`、`REQ-SEL-005`、`REQ-SEL-006`、`REQ-SEL-007`、`REQ-SEL-008` | 自动化 | `tests/role_metadata_runtime_tests.rs`、constraint target/validation trybuild |
| SEQ | `REQ-SEQ-001`、`REQ-SEQ-002`、`REQ-SEQ-003`、`REQ-SEQ-004`、`REQ-SEQ-005`、`REQ-SEQ-006`、`REQ-SEQ-007` | 自动化 | `rs-model-metadata/tests/constraint/sequence_constraint_tests.rs`、derive constraint trybuild |
| SER | `REQ-SER-001`、`REQ-SER-002`、`REQ-SER-003`、`REQ-SER-004`、`REQ-SER-005`、`REQ-SER-006`、`REQ-SER-007`、`REQ-SER-008` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/ui_roles/fail/keep_serializing.rs` |
| SYS | `REQ-SYS-001`、`REQ-SYS-002`、`REQ-SYS-003`、`REQ-SYS-004`、`REQ-SYS-005`、`REQ-SYS-006`、`REQ-SYS-007`、`REQ-SYS-008`、`REQ-SYS-009`、`REQ-SYS-010`、`REQ-SYS-011` | 自动化 + 审查 | runtime fixtures、全 workspace check、CI/style/doc scripts |
| TIME | `REQ-TIME-001`、`REQ-TIME-002`、`REQ-TIME-003`、`REQ-TIME-004` | 自动化 | `rs-model-metadata/tests/constraint/temporal_constraint_tests.rs`、derive constraint trybuild |
| TXT | `REQ-TXT-001`、`REQ-TXT-002`、`REQ-TXT-003`、`REQ-TXT-004`、`REQ-TXT-005`、`REQ-TXT-006`、`REQ-TXT-007`、`REQ-TXT-008`、`REQ-TXT-009`、`REQ-TXT-010`、`REQ-TXT-011`、`REQ-TXT-012` | 自动化 | `rs-model-metadata/tests/constraint/text_constraint_tests.rs`、derive runtime/trybuild |
| UNQ | `REQ-UNQ-001`、`REQ-UNQ-002`、`REQ-UNQ-003`、`REQ-UNQ-004`、`REQ-UNQ-005`、`REQ-UNQ-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、metadata resolver tests |
| VAL | `REQ-VAL-001`、`REQ-VAL-002`、`REQ-VAL-003`、`REQ-VAL-004`、`REQ-VAL-005`、`REQ-VAL-006`、`REQ-VAL-007`、`REQ-VAL-008` | 自动化 | `tests/role_metadata_runtime_tests.rs`、`tests/role_pipeline_tests.rs` |
| VAR | `REQ-VAR-001`、`REQ-VAR-002`、`REQ-VAR-003`、`REQ-VAR-004`、`REQ-VAR-005`、`REQ-VAR-006` | 自动化 | `tests/role_metadata_runtime_tests.rs`、metadata role tests |
| VLD | `REQ-VLD-001`、`REQ-VLD-002`、`REQ-VLD-003`、`REQ-VLD-004`、`REQ-VLD-005`、`REQ-VLD-006`、`REQ-VLD-007`、`REQ-VLD-008`、`REQ-VLD-009`、`REQ-VLD-010` | 自动化 | `tests/role_metadata_runtime_tests.rs`、validator overflow/constraint trybuild |

## 完成判定

任何需求变更都必须同步更新本台账和对应测试。过程宏输入总性、hidden ABI、Property 合并、泛型 Enum overlay、resolver 图约束和 Projection producer 属于阻断级验证面；这些测试失败时不得发布。下游 131 个真实声明的覆盖边界见 [rs-platform 模型声明基线](rs-platform-model-baseline.zh_CN.md)。
