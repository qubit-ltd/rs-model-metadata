# `qubit-model-derive` Requirements

This document is the English companion to the [Chinese requirements](rs-model-derive-requirements.zh_CN.md).

## Stable contract

The six public attribute macros—`Entity`, `Projection`, `Model`, `Enum`, `Value`, and `ModelImpl`—generate reflection and model metadata through the versioned runtime facade. Their public metadata roles and property metadata are protected by runtime and UI regression tests.

## Diagnostics and validation

The macro reports recoverable, independent declaration errors together. Duplicate singleton options, invalid role/field combinations, invalid text bounds, invalid explicit enum variant names, and overflowing generic constant literals are diagnosed at their source spans. Cross-model target and property resolution remains the resolver's responsibility.

## Compatibility boundaries

`#[variant(name = "...")]` controls only the canonical model name; it does not rename the Rust variant or Serde representation. `ModelImpl` accepts contract-conforming public methods in an inherent implementation block. Generated private symbols use stable fingerprints to avoid collisions between distinct target paths.

## Evidence

The current test mapping is maintained in the [Chinese coverage ledger](rs-model-derive-requirements-coverage.zh_CN.md). The final architecture is described in the [Chinese final design](rs-model-derive-final-design.zh_CN.md).
