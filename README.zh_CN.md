# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Rust 规则提供类型安全的验证契约，也支持按稳定规则 ID 配置和查找验证器。它帮助库和应用作者区分业务值不符合要求与绑定、执行失败，并把规则选择和诊断汇总方式交给调用方。

## 安装

```toml
[dependencies]
qubit-validator = "0.1"
```

## 快速开始

例如，表单处理逻辑可以先绑定已注册的 `NonBlank` 规则，再重复检查用户填写的显示名称。仓库中的完整可运行示例 [`examples/local_registry.rs`](examples/local_registry.rs) 会定义并注册规则，分别验证有效和无效输入，并检查产生的违规代码。

```bash
cargo run --example local_registry --locked
```

示例会确认 `"Ada"` 通过验证，空白文本产生 `text.blank` 违规；同时演示依赖顺序错误和参数重复读取错误。启用 `inventory` 后，还会验证进程级注册。

## 为什么需要这个项目

应用常常先直接调用验证函数，随后才需要按运行时选择规则、传入参数和有序依赖，并生成结构化诊断。本 crate 保留普通 Rust 类型化规则，同时提供清晰的绑定边界，供需要配置化执行的代码使用。

## 核心能力

- `Validator<T, C>` 支持带领域错误的直接验证。
- 预备验证器和已绑定验证器支持可复用的配置化执行。
- 静态签名与描述符声明输入形状和依赖契约。
- 局部注册表按稳定验证器 ID 确定性查找规则。
- 违规项、跳过结果、路径、安全参数和有界报告均采用结构化类型表示；报告由 `ValidationReport::record_outcome` 汇总。
- 类型化文本规则和依赖感知规则可分别使用简单或 context-aware adapter。

本 crate 不发现对象属性、不遍历对象图、不编译模型路径、不调度规则组，也不负责消息本地化。调用方负责选择验证值、执行规则并决定如何展示诊断。

## Feature 与安全边界

默认不启用任何 feature。直接验证、适配器、描述符和局部 `ValidatorRegistry` 都可直接使用。只有需要通过 `register_validator!` 与 `ValidatorRegistry::try_global` 进行进程级注册时才启用 `inventory`。

依赖以有序槽位声明。context-aware adapter 执行前，绑定边界会检查槽位顺序、类型和可选性。`ExecutionError` 只保存结构化类别和安全元数据，不保留 source error。违规参数不得包含被拒绝的输入。由于先决条件失败而跳过时，前置违规项保存在该 skipped entry 中，不会重复计入报告顶层违规列表。报告的违规项限额同时计算保留的顶层违规项和先决条件证据；`record_outcome` 会给违规项相对路径添加出现路径前缀。

先决条件证据保留原始失败位置的绝对路径。规则准备层只返回 `Valid` 或 `Invalid`；输入缺失或先决条件失败时，由调用方构造跳过结果。`ValidationReport::failures()` 先遍历顶层违规项，再按 skipped entry 顺序遍历先决条件证据；它保留重复项，不承诺全局出现顺序，迭代数量等于 `failure_count()`。字段路径只接受静态声明名称，运行时 map 位置使用 `MapEntry` 表示。

## 延伸阅读

- [用户指南](doc/user_guide.zh_CN.md)
- [设计文档](doc/design.zh_CN.md)
- [API 文档](https://docs.rs/qubit-validator)
- [English README](README.md)
- [English User Guide](doc/user_guide.md)
- [English Design](doc/design.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-validator](https://github.com/qubit-ltd/rs-validator)
