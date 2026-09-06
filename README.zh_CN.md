# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Qubit 应用提供类型安全的验证 trait、结构化验证报告和不可变注册表。

## 安装

```toml
[dependencies]
qubit-validator = "0.1"
```

## 快速开始

为规则实现带有显式类型化上下文的根导出 trait `Validator<T, C>`。需要由模型元数据调用的规则，可以预先构造为 `PreparedValidator`，再组装进局部 `ValidatorRegistry`；直接验证仍然是普通的类型化 Rust 调用。

```rust,ignore
use qubit_validator::Validator;
use std::convert::Infallible;

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = Infallible;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        let _ = value;
        Ok(())
    }
}
```

## 能力

- 带领域错误的类型化 validator。
- 借用的参数和依赖值。
- 不使用 `unsafe` 的安全类型擦除调用。
- 带有安全路径和参数的结构化 `Violation` 与 `ValidationReport`。
- 确定性的局部注册表；可选 `inventory` feature 提供进程级发现。

## Feature

默认 feature 集为空。启用 `registry` 可使用显式 descriptor 和 registration API；需要通过
`register_validator!` 进行进程级注册时启用 `inventory`，它会同时启用 `registry`。需要隔离测试或
维护多套规则时，应直接构造局部 `ValidatorRegistry`。

## 限制

本 crate 不发现模型属性，也不调度验证。模型路径编译和 `ValidationPlan` 执行由
`qubit-model-metadata` 负责；本 crate 只提供类型化规则契约、绑定原语、报告和注册表。运行故障和
业务违规分开表示，因此缺少规则或未启用 feature 时不会被误判为输入合法。

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
