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
- 类型化文本规则和依赖感知规则可分别使用简单或 context-aware adapter；
  依赖闭包适配器支持区分违规结果与执行错误。

直接调用者可使用 `BoundValidator::validate_named` 按签名名称绑定依赖，避免同类型槽位被静默互换。对于绑定阶段已经核验顺序的调用方，`validate` 仍是有序快速入口。局部注册表和可选 inventory 注册可共用每条内置规则的同一个 `ValidatorRegistration` 常量。

本 crate 不发现对象属性、不遍历对象图、不编译模型路径、不调度规则组，也不负责消息本地化。调用方负责选择验证值、执行规则并决定如何展示诊断。

## Feature 与安全边界

默认不启用任何 feature。直接验证、适配器、描述符和局部 `ValidatorRegistry` 都可直接使用。只有需要通过 `register_validator!` 与 `ValidatorRegistry::try_global` 进行进程级注册时才启用 `inventory`。

依赖以签名上的有序槽位声明。绑定所选签名时，会直接把该签名的依赖规格写入 bound validator；模型元数据负责核验实际依赖声明，执行时检查槽位值形状。`ExecutionError` 只通过显式可信诊断入口 `trusted_source()` 保留并读取拥有型原因；普通格式化和 `Error::source()` 不暴露原因文本。违规参数不得包含被拒绝的输入。先决条件失败的跳过项通过不透明 `FailureId` 引用已记录违规项，因此原始失败只计数和展示一次。`record_outcome` 返回 `RecordedOutcome`，包含收集是否完整以及该 occurrence 保留的失败 ID。引用失败不占用 `max_violations`；`max_skipped` 限制跳过 occurrence 数量。`ValidationReport::failures()` 只遍历原始违规项。参数的 `Debug` 输出会隐藏名称和值。

当规则需要返回多条违规，或需要区分数据无效与执行失败时，可使用
`prepare_text_with_context` 或 `prepare_typed_with_context`。报告要求
`record_outcome` 按非递减 occurrence 调用；同一 occurrence 可以记录多次。

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
