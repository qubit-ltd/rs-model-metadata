# 覆盖率测量

[English](coverage_measurement.md)

本项目保留既有覆盖率门槛，不设置逐文件豁免。`ci-check.sh` 和 `coverage.sh`
额外使用 `-C link-dead-code=yes`，使固定的 Rust 1.94.0 与
cargo-llvm-cov 0.8.6 能保留不同测试可执行文件中的覆盖率映射。

## 使用这一选项的原因

已有图查询测试调用了 `ResolvedProjectionSource::target` 并通过断言。原始合并
profile 和该测试可执行文件的独立报告均记录一次调用，但所有测试可执行文件的
联合报告记录为零。清理覆盖率构建缓存后，结果没有改变。

独立的[复现 fixture](../tests/fixtures/coverage_mapping/Cargo.toml) 不依赖本库或
第三方依赖：第一个测试程序调用普通读取方法，另一个同时调用普通方法和
`inline(always)` 方法。联合报告可能丢失内联方法的执行计数；保留链接代码后，
该计数恢复。fixture 还包含从未被调用的方法，它必须保持未覆盖，因此正确的
函数覆盖率为 **2/3**。

LLVM 曾修复过[相关的 unused mapping 问题](https://github.com/llvm/llvm-project/pull/107661)。
本项目采用该选项的依据是本地复现，不能据此断言与早先 LLVM 问题的根因完全相同。
[Rust 编译器文档](https://doc.rust-lang.org/rustc/codegen-options/index.html#link-dead-code)
一般不推荐使用这个选项，因此这里只用于 CI 和覆盖率测量；固定工具版本更新时，
应重新评估是否还需要它。

## 复现与移除条件

从仓库根目录执行以下命令。第一条命令执行前应取消设置 `RUSTFLAGS` 和
`CARGO_ENCODED_RUSTFLAGS`：

```bash
cargo +1.94.0 llvm-cov --manifest-path tests/fixtures/coverage_mapping/Cargo.toml --json --output-path target/coverage-mapping-default.json
RUSTFLAGS='-C link-dead-code=yes' cargo +1.94.0 llvm-cov --manifest-path tests/fixtures/coverage_mapping/Cargo.toml --json --output-path target/coverage-mapping-retained.json
```

检查报告中 `src/record.rs` 的文件汇总和三个方法的计数；相同源码位置的不同实例化
需要合并观察。第二份报告必须记录两个读取方法，并使 `never_called` 保持零。
这个故意保留未覆盖方法的 fixture 位于生产 workspace 之外，不能代替生产覆盖率检查。

两个入口共同加载 `scripts/coverage-rustflags.sh`，保留 Cargo 编码标志的优先级
和参数边界，也保留普通编译标志。由于 vendor CI runner 直接调用自己的 coverage
脚本，本地 CI 会把该选项传给所有子构建。GitHub 覆盖率任务调用项目的
`coverage.sh`，因此使用相同选项。普通 Cargo 命令和 release benchmark 不加载
该 helper；CI 二进制文件可能因此变大。

`project-ci-check.sh` 对两个入口执行进程边界测试，包括参数透传与 vendor
失败的退出码；同时检查当前双语文档的本地目标与 Markdown 标题锚点，并跟随
仓库内的文档链接。历史归档作为遍历终点：入口文件必须存在，但不重新验证其
历史外链。旧的源码 `:line` 链接只检查文件存在，不校验历史行号范围。
可执行 Markdown 示例由常规 Cargo 测试矩阵单独运行。
工具链更新后，应在不设置该选项的情况下重新运行 fixture 和
完整 workspace 覆盖率。只有两者都能正确保留已执行映射时，才一起移除 helper
和两个入口中的加载逻辑。不得通过改写报告、降低门槛或新增文件豁免抵消误差。
