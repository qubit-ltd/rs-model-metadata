# Coverage measurement

[简体中文](coverage_measurement.zh_CN.md)

The project keeps its existing coverage gates and has no per-file exemptions.
The `ci-check.sh` and `coverage.sh` entry points additionally use
`-C link-dead-code=yes` to preserve coverage mappings across test executables
with the pinned Rust 1.94.0 and cargo-llvm-cov 0.8.6 tools.

## Why this option is present

An existing graph query test called `ResolvedProjectionSource::target` and
passed its assertion. Both the raw merged profile and a report for that test
executable recorded one call. A combined report for all test executables
recorded zero. Cleaning the coverage build did not change this result.

The independent [fixture](../tests/fixtures/coverage_mapping/Cargo.toml)
reproduces the problem without this library or any external dependency. One
test executable calls an ordinary read; another calls that read and an
`inline(always)` read. The combined report can lose the inline read's execution
count. Retaining code restores the count. The fixture also contains an uncalled
method: it must remain uncovered, so correct function coverage is **2/3**.

LLVM has addressed a [related unused-mapping problem](https://github.com/llvm/llvm-project/pull/107661).
The local reproduction is the evidence for this project's option; it does not
establish that the earlier LLVM issue has exactly the same cause.
The [Rust compiler documentation](https://doc.rust-lang.org/rustc/codegen-options/index.html#link-dead-code)
does not recommend this option in general. Its use here is limited to CI and
coverage measurement and should be reconsidered when the pinned tools change.

## Reproduce and retire the option

Run from the repository root, with `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS`
unset for the first command:

```bash
cargo +1.94.0 llvm-cov --manifest-path tests/fixtures/coverage_mapping/Cargo.toml --json --output-path target/coverage-mapping-default.json
RUSTFLAGS='-C link-dead-code=yes' cargo +1.94.0 llvm-cov --manifest-path tests/fixtures/coverage_mapping/Cargo.toml --json --output-path target/coverage-mapping-retained.json
```

Inspect the `src/record.rs` file summary and the three method counts, aggregating
instantiations at the same source position. The second report must count both
reads and leave `never_called` at zero. This deliberately incomplete fixture is
outside the production workspace and is not a replacement for its coverage run.

Both entry points source `scripts/coverage-rustflags.sh`. Encoded Cargo flags
retain precedence and argument boundaries; plain flags are preserved. Local CI
passes the option to all of its child builds because the vendor CI runner
invokes its own coverage script. The GitHub coverage job calls the project's
`coverage.sh` and receives the same option. Ordinary Cargo commands and release
benchmarks do not load this helper. CI binaries may be larger as a result.

`project-ci-check.sh` runs process-boundary tests for both wrappers, including
argument forwarding and vendor failures. It also checks active bilingual
documentation destinations and Markdown heading anchors, following local
document links within the repository. Historical archive documents are leaves:
their entry files must exist, but historical outgoing links are not revalidated.
Legacy source `:line` links check file existence, not historical line bounds.
Executable Markdown examples run separately in the regular Cargo test matrix.
After a toolchain update, repeat the
fixture and full workspace coverage without the option. Remove the helper and
its wrapper wiring together only when both correctly retain executed mappings.
Never compensate by editing report counts, lowering gates, or adding file
exemptions.
