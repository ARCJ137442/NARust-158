# vNARS provider

`narust_158_vnars_provider` 是 NARust-158 的 Rust 原生适配入口，不是新的推理内核。
它直接启动现有的 `LauncherAlpha` / `InferenceEngine`，在 stdin/stdout 上实现
`vnars.provider/1` JSON Lines 协议。

## 启动

```bash
cargo build --bin narust_158_vnars_provider
./target/debug/narust_158_vnars_provider
```

进程先输出一条 `ready` 帧，然后逐行读取 `command` 和 `close` 帧。命令输出结束后
发送 `{"kind":"command_end","boundary":"sealed"}`，因此调用方不需要依赖静默窗口
或解析推理输出。

## 命令映射

| provider 命令 | NARust-158 实现 |
|---|---|
| `input` | `Cmd::NSE(TaskJson)`，TaskJson 与 `narsese` lexical `Task` 双向转换 |
| `advance` | `Cmd::CYC(cycles)` |
| `configure` | `Cmd::VOL(volume)`，范围限制为 0–100 |
| `reset` | `Cmd::RES { target: "" }` |
| `register_operation` | 当前返回带请求 id 的错误 |
| `control` | 当前返回带请求 id 的错误 |

`IN`、`OUT`、`ANSWER` 如果带可解析的 Narsese，会额外给出结构化 `task`；其余输出保留
原始 `type` 和 `content`。协议接受的最大输入帧为 1 MiB。

## 身份与能力边界

- 只新增了 `src/vnars_provider.rs`、provider bin 和测试，以及库导出与 Cargo bin 声明；
  未修改推理内核实现。
- NARust-158 的 Cargo 依赖版本与特性未因本适配层升级；适配层复用仓库已经声明的
  `narsese` lexical 类型和 NAVM 接口。
- provider 在进程内运行现有 NAVM runtime，不经过 Python 或 shell 文本中转。
- `state_snapshotting`、`state_restoration`、`runtime_cloning` 仍是 unsupported；
  不要从本适配层推断 vNARS 已具备这些能力。
- 输出中的 NAVM `budget` 会随 NARust-158 的真实输出保留；这与 NARust-o 的
  TaskJson 预算字段省略行为不同，比较时不能只按 JSON 深度相等判断语义。

## 验证

```bash
cargo test --test vnars_provider
cargo test --all-targets
cargo build --all-targets
cargo fmt --all -- --check
```

2026-11-04 在 Termux/aarch64 上的 provider 测试为 7/7 通过；最近一次已记录的全目标
测试中库测试为 361 passed / 1 ignored / 0 failed。仓库现有 clippy 债务不属于本适配层：
`cargo clippy --all-targets -- -D warnings` 仍会因既有库代码与既有测试告警失败，
不能把该命令写成通过。提交前已重新运行上述命令，以最终日志为准。
