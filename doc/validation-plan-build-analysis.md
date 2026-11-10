# Validation plan build scaling analysis

[简体中文](validation-plan-build-analysis.zh_CN.md)

> **Historical experiment:** This bounded performance run predates the current fixed validation-plan build limits. Its measurements below describe that earlier implementation only; they are not performance measurements of the current root-scoped walker or evidence that current builds are unbounded. See the [current build limits and diagnostics](user_guide.md#advanced-usage-stopping-policies-and-work-budgets).

## Question and method

This bounded experiment checked whether plan construction did work that grew
beyond the occurrences the plan had to retain. It compares a shared binary model
tree with a single chain. Each leaf has one `non_blank` text declaration. At
depths 1, 4, 8, and 10, the binary shapes produce 2, 16, 256, and 1,024
occurrences, while the chain produces one occurrence at each depth. Graph
resolution and validator setup happen outside the timed loop; the timed
operation is `ValidationPlan::build`.

The benchmark asserted the resolved unique model count and plan binding count
before timing. `tests/validation/validation_plan_tests.rs` separately builds a
root with the same constrained child type on two fields, asserts two bindings
and stable `left.value`, `right.value` violation order, then verifies that a
valid left value produces only the `right.value` violation.

## Reproduction and environment

```text
cargo bench --locked --features validation --bench model_pipeline -- plan_build_scaling
```

The measured run used the release profile, Rust `1.94.0` (`4a4ef493e`),
`x86_64-unknown-linux-gnu`, on an Intel Xeon E5-2680 v4 at 2.40 GHz. Criterion
used 10 samples per case, a 3-second warmup and a 1-second measurement window.
The host had other Cargo jobs running, so these timings are estimates rather
than a stable performance gate. Peak RSS was not measured.

## Measurements

Criterion's reported estimate and confidence interval:

| Shape | Depth | Unique types | Bindings | Plan build time |
| --- | ---: | ---: | ---: | ---: |
| Binary | 1 | 2 | 2 | 7.78–8.05 µs (estimate 7.85 µs) |
| Binary | 4 | 5 | 16 | 72.53–77.00 µs (estimate 74.57 µs) |
| Binary | 8 | 9 | 256 | 1.381–1.394 ms (estimate 1.389 ms) |
| Binary | 10 | 11 | 1,024 | 6.090–8.438 ms (estimate 7.096 ms) |
| Chain | 1 | 2 | 1 | 5.03–5.07 µs (estimate 5.05 µs) |
| Chain | 4 | 5 | 1 | 10.32–10.45 µs (estimate 10.38 µs) |
| Chain | 8 | 9 | 1 | 18.30–18.35 µs (estimate 18.33 µs) |
| Chain | 10 | 11 | 1 | 22.37–22.78 µs (estimate 22.54 µs) |

The binary cases grow with the 2^depth occurrences retained by the output;
there is no measurement showing a superlinear cost relative to those bindings.
The chain shows a smaller increase with the number of unique types even though
its binding count stays at one, consistent with traversing a deeper graph.
Neither result isolates allocations or proves that every repeated metadata
read is necessary, but the experiment found no actionable redundant work with
measurable benefit that preserves the existing occurrence contract.

## Downstream check and conclusion

`rs-platform/modules/model-runtime/src/lib.rs` builds a root graph on the first
successful validation request and caches the resulting plan by root `TypeId`;
subsequent requests reuse it. The platform `Person` model contains three
optional `CredentialInfo` fields, whose `number` field has a validator, so the
plan appropriately retains three path-specific occurrences. This is bounded
fan-out, not a deep shared binary shape. The much wider synthetic tree is a
stress shape, not evidence of a production failure.

Historical conclusion: path expansion has an inherent cost when distinct paths
require distinct bindings. This experiment found no actionable redundant work
under the implementation it measured. Its timing results do not characterize the
current walker. The current implementation has fixed construction limits, which
are separate from `ValidationOptions` instance-execution budgets; see the
[current user guide](user_guide.md#advanced-usage-stopping-policies-and-work-budgets).
Applications should build a plan once and reuse it, as `ModelRuntime` does.
