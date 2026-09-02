# `qubit-model-derive` Final Design

This document is the English companion to the [Chinese final design](rs-model-derive-final-design.zh_CN.md).

The crate processes a declaration through explicit parsing, normalization, validation, and expansion stages. The declaration intermediate representation is private to the macro implementation and carries source locations for diagnostics. A shared diagnostic accumulator combines independent recoverable failures before returning compiler errors.

Field and role rules are validated before metadata code generation. The generated code uses the versioned reflection facade. Generic metadata represents supported symbolic types and constants; overflowing integer literals are rejected instead of being silently replaced. `ModelImpl` validates property-method contracts and gives generated adapters deterministic, fingerprinted identifiers.

The authoritative behavioral evidence is the test suite and its UI snapshots. Historical discussion notes are archival material, not a specification source.
