# Qubit Validator（`rs-validator`）

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Rust 应用提供可复用的类型化规则、规则绑定和结构化校验报告。以 `POST /users` 为例：后端在写入用户资料前必须拒绝空白用户名和格式错误的邮箱。这些检查应以 `Validator` 实现的形式落在本 crate，并挂上稳定 ID。采用字段注解时，领域类型上的注解只引用规则 ID；可选的 `qubit-model-metadata` 集成负责绑定规则并遍历对象。规则集中在注册表里，就不必在每个接口里复制同一套判断。

## 用户注册场景

用户在注册页面填写用户名和邮箱，REST 接口将请求解析为 `User`，再交给注册服务。服务启动时注册并绑定 `app.non_blank` 与 `app.email`。每次请求到来时，服务分别校验 `username` 和 `email` 字段，并将结果写入报告：输入不合规则时，报告包含字段路径和稳定违规代码，且不会写入仓储；报告完整且全部通过时才允许写入。HTTP 层可将字段违规映射为客户端错误，将存储或校验执行故障映射为服务错误。路由和持久化由应用提供。需要按字段注解遍历对象的应用，可采用下文介绍的可选 `qubit-model-metadata` 集成。

## 安装

Rust 1.94 或更高版本即可使用。直接调用规则和本地注册表只需安装本 crate：

<!-- validator-quick-start:manifest -->
```toml
[dependencies]
qubit-validator = "0.1"
```

## 快速开始

下面的完整程序用用户注册流程演示如何注册规则、在应用启动时绑定规则，并在写入仓储前检查每个字段。`CountingRepository` 只用于观察拒绝和写入分支；实际服务可换成自己的仓储实现。

<!-- validator-quick-start:source -->
```rust
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_validator::{
    ArgumentReader, BoundValidationContext, InputType, NamedValidationArgument, PreparedValidator,
    RegistrationSource, ValidationPath, ValidationReport, ValidationValue, Validator,
    ValidatorDescriptor, ValidatorId, ValidatorRegistration, ValidatorRegistry, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

#[derive(Debug)]
struct RuleError(&'static str);
impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for RuleError {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = RuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(RuleError("text is blank"))
        } else {
            Ok(())
        }
    }
}

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}
static NON_BLANK_SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(
    InputType::Text,
    &[],
    prepare_non_blank,
)];
static NON_BLANK_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(NON_BLANK_SIGNATURES);
static NON_BLANK_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.non_blank"),
    &NON_BLANK_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

struct AsciiEmail;
impl Validator<str> for AsciiEmail {
    type Error = RuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(RuleError("invalid email shape"))?;
        let domain = parts.next().ok_or(RuleError("invalid email shape"))?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || !domain.contains('.')
            || domain.split('.').any(str::is_empty)
        {
            Err(RuleError("invalid ASCII email shape"))
        } else {
            Ok(())
        }
    }
}
fn prepare_email(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, qubit_validator::BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(AsciiEmail, |_| {
        ViolationDraft::new(ViolationCode::new("text.email"))
    }))
}
static EMAIL_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_email)];
static EMAIL_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(EMAIL_SIGNATURES);
static EMAIL_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.email"),
    &EMAIL_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

struct User {
    username: String,
    email: String,
}
struct CountingRepository(Cell<usize>);
impl CountingRepository {
    fn insert(&self, _user: &User) {
        self.0.set(self.0.get() + 1);
    }
}

fn validate_and_insert(
    user: &User,
    non_blank: &qubit_validator::BoundValidator,
    email: &qubit_validator::BoundValidator,
    repository: &CountingRepository,
) -> Result<ValidationReport, Box<dyn Error>> {
    let mut report = ValidationReport::new();
    for (index, (field, value, rule)) in [
        ("username", user.username.as_str(), non_blank),
        ("email", user.email.as_str(), email),
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = rule.validate(
            ValidationValue::Text(value),
            &BoundValidationContext::new(&[]),
        )?;
        report.record_outcome(index, ValidationPath::root().with_field(field), outcome)?;
    }
    if !report.is_truncated() && report.is_valid() {
        repository.insert(user);
    }
    Ok(report)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Register and bind once during application startup.
    let registry =
        ValidatorRegistry::from_registrations([&NON_BLANK_REGISTRATION, &EMAIL_REGISTRATION])?;
    let non_blank = registry.bind("app.non_blank", InputType::Text, &[])?;
    let email = registry.bind("app.email", InputType::Text, &[])?;
    let repository = CountingRepository(Cell::new(0));

    let invalid = User {
        username: "   ".into(),
        email: "wrong".into(),
    };
    let report = validate_and_insert(&invalid, &non_blank, &email, &repository)?;
    assert!(!report.is_truncated());
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].code().as_str(), "text.blank");
    assert_eq!(report.violations()[0].path().render(), "username");
    assert_eq!(report.violations()[1].code().as_str(), "text.email");
    assert_eq!(report.violations()[1].path().render(), "email");
    assert_eq!(repository.0.get(), 0);

    let valid = User {
        username: "ada".into(),
        email: "ada@example.com".into(),
    };
    let report = validate_and_insert(&valid, &non_blank, &email, &repository)?;
    assert!(!report.is_truncated());
    assert!(report.is_valid());
    assert_eq!(repository.0.get(), 1);
    Ok(())
}
```

运行 `cargo run`。无效请求的报告包含 `username: text.blank` 和 `email: text.email` 两条违规，仓储写入次数为零；有效请求写入一次。邮箱规则只检查演示用的 ASCII 邮箱形态，不验证邮箱归属。

## 模型注解集成

`qubit-validator` 负责定义、准备、注册和执行规则；对象字段的遍历由 `qubit-model-metadata` 提供。原先基于 `#[Model]` 与 `ValidationPlan` 的示例属于这条可选的进阶路径。

在同一 checkout 中采用以下目录布局时，应用的 `Cargo.toml` 可使用这些本地依赖。此示意不是首次使用 `qubit-validator` 的安装条件：

```text
<checkout>/rust-common/{rs-validator,rs-model-metadata,rs-reflect,rs-validation-rules}
<checkout>/app
```

在 `<checkout>/app/Cargo.toml` 中：

```toml
[dependencies]
qubit-validator = { version = "0.1", path = "../rust-common/rs-validator" }
qubit-model-metadata = { path = "../rust-common/rs-model-metadata", features = ["validation"] }
qubit-model-derive = { path = "../rust-common/rs-model-metadata/derive" }
qubit-reflect = { path = "../rust-common/rs-reflect" }
```

当前 checkout 中，`qubit-model-metadata` 和 `qubit-model-derive` 均设置为 `publish = false`。详见[模型校验用户指南](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.zh_CN.md)。

## 核心能力

- `Validator<T, C>` 表示可直接调用的类型化规则，并保留规则自己的领域错误。
- `ValidatorSignature`、`ValidatorDescriptor` 和 `ValidatorRegistry` 按稳定 ID 选择、准备规则；`BoundValidator` 核验选中的输入和依赖形状。
- `ValidationOutcome`、`Violation` 和 `ValidationReport` 区分输入不合规则与绑定、执行故障，并汇总字段问题。
- 适配器和 `ArgumentReader` 支持带参数或命名依赖的可复用规则；可选的 `inventory` 用于发现已链接的自定义注册项。

`qubit-validator` 不会检查模型字段或自动调度规则。快速开始展示直接调用路径：启动时绑定规则，由应用选择字段执行校验并组装报告，再据此决定是否保存。需要字段注解和自动遍历对象的应用，可使用可选的 [模型注解集成](#模型注解集成)。绑定与报告 API 见[验证器用户手册](doc/user_guide.zh_CN.md)；计划构建、可执行字段形状、部分结果和限额见[模型校验手册](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.zh_CN.md#实战场景续校验一个用户实例)。

## 延伸阅读

- [验证器用户手册](doc/user_guide.zh_CN.md) · [设计文档](doc/design.zh_CN.md) · [API 文档](https://docs.rs/qubit-validator)
- [模型校验手册](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.zh_CN.md) · [字段声明手册](https://github.com/qubit-ltd/rs-model-metadata/blob/main/derive/doc/user_guide.zh_CN.md)
- [English README](README.md) · [English user guide](doc/user_guide.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
.infra/bin/ci-check.sh

# 检查代码覆盖率
.infra/bin/coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试。本仓库以 `.infra/bin/align-ci.sh` 为格式化命令，以 `.infra/bin/ci-check.sh` 为最终检查。默认 `cargo fmt --check` 可能报告与项目格式不一致的差异；不要据此批量改写全库文件。

## 作者

**Haixing Hu**

仓库地址：[https://github.com/qubit-ltd/rs-validator](https://github.com/qubit-ltd/rs-validator)
