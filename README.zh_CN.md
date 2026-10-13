# Qubit Validator（`rs-validator`）

[![Rust CI](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validator/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validator/coverage-badge.json)](https://qubit-ltd.github.io/rs-validator/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validator.svg?color=blue)](https://crates.io/crates/qubit-validator)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-validator` 为 Rust 应用提供可复用的类型化规则、规则绑定和结构化校验报告。以 `POST /users` 为例：后端在写入用户资料前必须拒绝空白用户名和格式错误的邮箱。这些检查应以 `Validator` 实现的形式落在本 crate，并挂上稳定 ID。领域类型字段上的注解只是引用这些 ID；由 `qubit-model-metadata` 完成绑定并遍历对象。规则集中在注册表里，就不必在每个接口里复制同一套判断。

## 用户注册场景

用户在注册页面填写用户名和邮箱，REST 接口将请求解析为 `User`，再交给注册服务。服务启动时注册两条自定义规则——`app.non_blank` 与 `app.email`——并根据 `User` 的字段声明构建可复用的校验计划。每次请求到来时，计划执行这些规则：用户名为空白或不满足 ASCII 邮箱形态，就返回字段路径和稳定违规代码，不调用用户仓储；两项都通过，才写入仓储。HTTP 层可将字段违规映射为客户端错误，将存储或校验执行故障映射为服务错误。路由和持久化由应用提供。

## 安装

只直接使用规则与局部注册表时，依赖 `qubit-validator = "0.1"` 即可。下面的快速开始还要通过字段注解应用这些规则，因此需要启用 `qubit-model-metadata` 的 `validation` 功能，并使用 `qubit-model-derive`：

```toml
[dependencies]
qubit-validator = "0.1"
qubit-model-metadata = { version = "0.2", features = ["validation"] }
qubit-model-derive = "0.2"
qubit-reflect = "0.2"
```

最低 Rust 版本为 1.94。

## 快速开始

以下程序完整展示注册时的校验与保存决策。将它放在上述应用项目的 `src/main.rs`。示例覆盖本 crate 公开面上的三步：实现 `Validator`、按稳定 ID 注册规则，再在 `User` 字段上引用这些 ID。`CountingRepository` 只是为了让“拒绝时不写、通过后写一次”可观察；接入 REST 服务时应换成应用自己的仓储实现。

```rust
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};
use qubit_model_metadata::validation::{ValidationBuildInputs, ValidationOptions, ValidationPlan};
use qubit_reflect::ReflectedRef;
use qubit_validator::{
    ArgumentReader, BindError, InputType, NamedValidationArgument, PreparedValidator,
    RegistrationSource, ValidationReport, Validator, ValidatorDescriptor, ValidatorId,
    ValidatorRegistration, ValidatorRegistry, ValidatorSignature, ViolationCode, ViolationDraft,
    prepare_text_validator,
};

// --- 自定义规则：非空白文本 ------------------------------------------------

#[derive(Debug)]
struct BlankText;
impl fmt::Display for BlankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text must not be blank")
    }
}
impl Error for BlankText {}

struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankText;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
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

static NON_BLANK_SIGNATURES: &[ValidatorSignature] =
    &[ValidatorSignature::new(InputType::Text, &[], prepare_non_blank)];
static NON_BLANK_DESCRIPTOR: ValidatorDescriptor =
    ValidatorDescriptor::new(NON_BLANK_SIGNATURES);
static NON_BLANK_REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("app.non_blank"),
    &NON_BLANK_DESCRIPTOR,
    RegistrationSource::new("registration", "rules", file!(), line!()),
);

// --- 自定义规则：ASCII 邮箱形态 --------------------------------------------

#[derive(Debug)]
struct BadEmail;
impl fmt::Display for BadEmail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text is not a valid ASCII email")
    }
}
impl Error for BadEmail {}

struct AsciiEmail;
impl Validator<str> for AsciiEmail {
    type Error = BadEmail;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(BadEmail)?;
        let domain = parts.next().ok_or(BadEmail)?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || !domain.contains('.')
            || domain.split('.').any(|label| label.is_empty())
        {
            Err(BadEmail)
        } else {
            Ok(())
        }
    }
}

fn prepare_email(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
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

// --- 领域类型：注解只引用已注册的 ID --------------------------------------

#[Model]
struct User {
    #[validator(id = "app.non_blank")]
    username: String,
    #[validator(id = "app.non_blank")]
    #[validator(id = "app.email")]
    email: String,
}

trait UserRepository {
    fn insert(&self, user: User) -> Result<(), Box<dyn Error>>;
}

enum RegistrationOutcome {
    Created,
    Rejected(ValidationReport),
}

fn register_user(
    plan: &ValidationPlan<'_>,
    repository: &dyn UserRepository,
    request: User,
) -> Result<RegistrationOutcome, Box<dyn Error>> {
    let report = plan.validate(ReflectedRef::new(&request), &ValidationOptions::default())?;
    if report.is_truncated() {
        return Err(std::io::Error::other("registration validation was incomplete").into());
    }
    if !report.is_valid() {
        return Ok(RegistrationOutcome::Rejected(report));
    }
    repository.insert(request)?;
    Ok(RegistrationOutcome::Created)
}

fn main() -> Result<(), Box<dyn Error>> {
    // 启动时构建一次计划，之后每次请求复用。
    let root = TypeMetadata::of::<User>();
    let roots = [root];
    let models = ModelRegistry::try_global()?;
    let graph = Arc::new(StructureResolver::for_roots(ResolveInputs {
        models: &models,
        roots: &roots,
    }).resolve()?);
    let validators = ValidatorRegistry::from_registrations([
        NON_BLANK_REGISTRATION,
        EMAIL_REGISTRATION,
    ])?;
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph,
        validators: &validators,
    })?;

    struct CountingRepository(Cell<usize>);
    impl UserRepository for CountingRepository {
        fn insert(&self, _user: User) -> Result<(), Box<dyn Error>> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
    }
    let repository = CountingRepository(Cell::new(0));
    let invalid = User { username: "   ".into(), email: "wrong".into() };
    let RegistrationOutcome::Rejected(report) = register_user(&plan, &repository, invalid)? else {
        panic!("invalid registration must be rejected");
    };
    assert_eq!(repository.0.get(), 0);
    for violation in report.violations() {
        println!("{}: {}", violation.path().render(), violation.code().as_str());
    }

    let valid = User { username: "ada".into(), email: "ada@example.com".into() };
    assert!(matches!(
        register_user(&plan, &repository, valid)?, RegistrationOutcome::Created
    ));
    assert_eq!(repository.0.get(), 1);
    Ok(())
}
```

运行 `cargo run`。无效请求打印 `username: text.blank` 和 `email: text.email`，仓储写入次数保持为零；合法请求使写入次数变为一。

`NonBlank` 与 `AsciiEmail` 是普通的 `Validator<str>` 实现。`prepare_text_validator` 把领域错误映射为稳定违规代码；描述符和 `ValidatorRegistration` 把准备好的规则发布为 `app.non_blank` 或 `app.email`。`User` 上的 `#[validator(id = "...")]` 只声明各字段要使用哪些已注册 ID，构造时并不会自动执行。`qubit-model-metadata` 的 `ValidationPlan::build` 读取这些声明并绑定对应注册项；`validate` 遍历受支持的字段，返回本 crate 的 `ValidationReport`。`#[text(non_blank)]` 这类内置约束可以不经自定义注册完成绑定；自定义 `#[validator(id = "...")]` 则必须在传入计划的注册表中提供同 ID 的注册项。

服务先检查 `is_truncated()`，避免用不完整的报告放行或向用户声称已列出所有问题。示例中的邮箱规则只检查简单的 ASCII 形态，不验证邮箱归属。用户名是否已被占用等依赖数据库的规则，由应用服务负责。诊断参数和日志不应包含被拒绝的原始输入。

## 核心能力

- `Validator<T, C>` 表示可直接调用的类型化规则，并保留规则自己的领域错误。
- `ValidatorSignature`、`ValidatorDescriptor` 和 `ValidatorRegistry` 按稳定 ID 选择、准备规则；`BoundValidator` 核验选中的输入和依赖形状。
- `ValidationOutcome`、`Violation` 和 `ValidationReport` 区分输入不合规则与绑定、执行故障，并汇总字段问题。
- 适配器和 `ArgumentReader` 支持带参数或命名依赖的可复用规则；可选的 `inventory` 用于发现已链接的自定义注册项。

`qubit-validator` 单独使用时不会检查 `User` 的字段或自动调度规则。应用可以直接绑定规则并自行组装报告；快速开始使用 `qubit-model-metadata` 作为可选的模型级执行路径，以读取字段注解。底层绑定与报告 API 见[验证器用户手册](doc/user_guide.zh_CN.md)；计划构建、可执行字段形状、部分结果和限额见[模型校验手册](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.zh_CN.md#实战场景续校验一个用户实例)。

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
