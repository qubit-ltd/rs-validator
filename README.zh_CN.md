# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Rust 应用提供一条小而明确的类型安全验证边界：既支持直接的类型化规则，也支持预构造的类型擦除规则、结构化违规结果和确定性的规则注册表。这样，输入不符合规则、规则绑定失败和执行失败可以被分别处理。

## 安装

```toml
[dependencies]
qubit-validator = "0.1"
```

## 快速开始

例如，应用需要拒绝空白用户名，同时希望把同一条规则交给准备好的验证边界执行。可以先实现类型化规则，再通过适配器生成结构化违规结果；过程中不需要把输入转成字符串或复制输入：

```rust,ignore
use qubit_validator::BoundValidationContext;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::Validator;
use qubit_validator::ValidationValue;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;

#[derive(Debug)]
struct BlankName;

impl std::fmt::Display for BlankName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("name is blank")
    }
}

impl std::error::Error for BlankName {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankName;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankName) } else { Ok(()) }
    }
}

let prepared = prepare_text_validator(NonBlank, |_| {
    ViolationDraft::new(ViolationCode::new("user.name.blank"))
});
let outcome = prepared
    .validate(ValidationValue::Text("  "), &BoundValidationContext::new(&[]))?;
assert!(matches!(outcome, PreparedOutcome::Invalid(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

类型化实现仍然是普通 Rust 代码；适配器则把失败转换为报告或注册表执行器可以继续处理的结构化违规。

## 能力

- 带领域错误的类型化 validator。
- 借用的参数和依赖值。
- 不使用 `unsafe` 的安全类型擦除调用。
- 带有安全路径和参数的结构化 `Violation` 与 `ValidationReport`。
- 确定性的局部注册表；可选 `inventory` feature 提供进程级发现。

## Feature

默认 feature 集为空。类型化 descriptor、registration 和局部注册表 API 始终可用；只有需要通过 `register_validator!` 进行进程级注册时才启用 `inventory`。需要隔离测试或维护多套规则时，应直接构造局部 `ValidatorRegistry`；需要限制报告规模时使用 `ValidationReport::with_limits`。

## 限制

本 crate 不发现模型属性，也不调度验证。模型路径编译和 `ValidationPlan` 执行由
`qubit-model-metadata` 负责；本 crate 只提供类型化规则契约、绑定原语、报告和注册表。运行故障和
业务违规分开表示，因此缺少规则或未启用 feature 时不会被误判为输入合法。

## 延伸阅读

- [English user guide](doc/user_guide.md) / [中文用户手册](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-validator)
- [English README](README.md)

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
Pull Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh`
对齐 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-validator](https://github.com/qubit-ltd/rs-validator)
