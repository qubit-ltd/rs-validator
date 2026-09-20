# qubit-validator

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Rust 应用提供类型安全的验证 trait、显式绑定、结构化验证结果和不可变验证器注册表。

## 安装

```toml
[dependencies]
qubit-validator = "0.1"
```

## 快速开始

下面的程序定义了类型化的 `NonBlank` 规则，将其领域错误映射为稳定代码
`text.blank`，把规则注册到局部注册表中，完成绑定后分别检查有效值和无效值。
这也是 [`examples/local_registry.rs`](examples/local_registry.rs) 的完整源码；运行
`cargo run --example local_registry` 即可执行。

```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

#[derive(Debug)]
struct BlankText;

impl fmt::Display for BlankText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("text must not be blank")
    }
}

impl Error for BlankText {}

struct NonBlank;

impl Validator<str> for NonBlank {
    type Error = BlankText;

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(BlankText)
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_non_blank,
)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

#[cfg(feature = "inventory")]
register_validator!(id = "text.non_blank.global", descriptor = &DESCRIPTOR);

static DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("minimum", InputType::Text, false),
    DependencySpec::new("maximum", InputType::Text, false),
];
static DEPENDENCY_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    DEPENDENCIES,
    prepare_non_blank,
)];
static DEPENDENCY_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(DEPENDENCY_SIGNATURES);

fn assert_dependency_order_is_checked() {
    let declared = [DEPENDENCIES[1], DEPENDENCIES[0]];
    let error = DEPENDENCY_DESCRIPTOR
        .bind(ValidatorId::new("text.dependent"), 0, &[], &declared)
        .expect_err("swapped dependency slots must fail during binding");
    assert_eq!(error.kind(), BindErrorKind::DependencyOrderMismatch);
}

fn assert_parameters_are_consumed_once() -> Result<(), BindError> {
    let args = [NamedValidationArgument::new(
        "limit",
        ValidationArgument::Unsigned(10),
    )];
    let mut reader = ArgumentReader::new(&args)?;
    assert_eq!(reader.required_u32("limit")?, 10);
    let error = reader
        .required_u32("limit")
        .expect_err("a parameter cannot be read twice");
    assert_eq!(error.kind(), BindErrorKind::ParameterAlreadyConsumed);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;

    assert!(registry.get("text.non_blank").is_some());
    let validator = registry.bind("text.non_blank", InputType::Text, &[], &[])?;
    let context = BoundValidationContext::new(&[]);

    let valid = validator.validate(ValidationValue::Text("Ada"), &context)?;
    assert_eq!(valid, ValidationOutcome::Valid);

    let invalid = validator.validate(ValidationValue::Text("   "), &context)?;
    let ValidationOutcome::Invalid(violations) = invalid else {
        panic!("blank text must be invalid");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code(), ViolationCode::new("text.blank"));

    assert_dependency_order_is_checked();
    assert_parameters_are_consumed_once()?;

    #[cfg(feature = "inventory")]
    {
        let global = ValidatorRegistry::try_global()?;
        assert!(global.get("text.non_blank.global").is_some());
    }

    Ok(())
}
```

## 为什么需要这个项目

应用中的验证往往始于直接函数调用，之后又需要按运行时选择规则、传入配置参数和依赖值，并提供可诊断信息。本 crate 让这些需求遵循同一套契约：规则仍是普通的类型化 Rust 代码，而显式适配器则为配置化执行提供安全的类型擦除边界。业务值无效与绑定失败或执行失败始终分开表示。

## 核心能力

- `Validator<T, C>` 支持带领域错误的直接类型化验证。
- 预备验证器和已绑定验证器支持可复用的配置化执行。
- 静态签名与描述符声明输入和依赖契约。
- 确定性的局部注册表使用稳定的验证器 ID 作为键。
- 结构化的违规项、跳过结果、路径、参数以及有界报告。
- 不使用 `unsafe`、也不克隆输入值的安全类型擦除。

## Feature

默认 feature 集为空。直接验证、描述符、注册以及局部 `ValidatorRegistry` 的构造无需启用任何 feature。只有需要通过 `register_validator!` 和 `ValidatorRegistry::try_global` 进行进程级注册时才启用 `inventory`。对于隔离测试、插件边界或同一进程中的多套规则，应优先使用局部注册表。

依赖按有序槽位声明和提供；依赖顺序、输入类型和可选性必须与所选签名完全一致。验证器参数由 `ArgumentReader` 解码一次；重复名称、未知名称、错误类型、有损转换以及重复读取都会被拒绝。

## 限制

本 crate 不发现对象属性、不编译模型路径、不遍历对象图，也不调度验证器组。调用方负责选择值和依赖、调用已绑定验证器并组装报告。

执行过程同步且使用借用。错误及其公共格式化内容不会携带或输出原始输入。受信任的展示代码如需路径字符串，必须显式调用 `ValidationPath::render`。

## 延伸阅读

- [用户指南](doc/user_guide.zh_CN.md)
- [设计](doc/design.zh_CN.md)
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
