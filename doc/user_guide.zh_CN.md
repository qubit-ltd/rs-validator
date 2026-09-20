# qubit-validator 用户指南

## 手册目标与读者

`qubit-validator` 面向同时需要常规类型化验证，以及通过稳定规则 ID 配置查找验证器的 Rust 库与应用。本指南假定读者了解 Rust trait 和错误处理的基础知识，重点介绍公共 API，不要求使用模型框架、代码生成器或调度器。

## 概念模型

API 有意区分以下六个概念：

| 概念 | 含义 |
| --- | --- |
| 直接类型化验证 | 使用具体 Rust 类型调用 `Validator<T, C>::validate`，并接收规则的领域错误。 |
| 预备验证器 | 配置参数完成解码后生成的可复用、类型擦除的规则实例。它返回 `PreparedOutcome` 草稿或执行错误。 |
| 已绑定验证器 | 与选定签名及稳定规则 ID 配对的预备验证器。它在执行前检查输入和有序依赖槽位。 |
| 验证结果 | 一次成功调用的结果：`Valid`、`Invalid` 或 `Skipped`。输入无效属于数据，不是执行失败。 |
| 执行错误 | 输入不匹配、缺少必需依赖或违反适配器契约等基础设施或契约失败，以 `Err` 返回。 |
| 报告 | 由调用方组装的 `ValidationReport`，汇总最终的 `Violation` 与 `SkippedValidation` 值，并可选择设置限制。 |

在直接调用时，规则可以保留领域专用错误；只有越过预备边界时，才会把该错误映射为稳定的 `ViolationDraft`。

## 场景：验证配置化的显示名称

贯穿本指南的场景使用 `NonBlank` 规则验证显示名称。直接类型化验证通过 `Result<(), BlankText>` 回答一个简单问题。配置化验证则将同一规则以 `text.non_blank` 注册，把 `BlankText` 映射为公共违规项代码 `text.blank`，再通过注册表调用规则。

完整可运行源码位于 [`examples/local_registry.rs`](../examples/local_registry.rs)。下一节也完整展示了这份源码，方便直接复制为程序。

## 安装与最小配置

如需直接验证和局部注册表，不必启用可选 feature，直接添加依赖：

```toml
[dependencies]
qubit-validator = "0.1"
```

默认 feature 集已经支持局部注册。以下是完整的 `examples/local_registry.rs` 程序：

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

运行这份权威示例源码：

```bash
cargo run --example local_registry --locked
```

## 核心工作流

1. 为类型化规则实现 `Validator<T, C>`。不需要运行时查找时，直接调用即可。
2. 使用 `prepare_text_validator` 或 `prepare_typed_validator` 适配规则。映射器把领域错误转换为带稳定代码和安全参数的 `ViolationDraft`。
3. 将准备函数放入一个或多个静态 `ValidatorSignature`。每个签名固定其接受的输入形状及有序依赖槽位。
4. 将签名放入静态 `ValidatorDescriptor`，再把描述符与 `ValidatorId` 和 `RegistrationSource` 关联起来。
5. 把注册冻结为局部注册表，查找或绑定规则，然后复用生成的 `BoundValidator`。
6. 每次调用时传入借用的 `ValidationValue` 和 `BoundValidationContext`。处理非穷尽的 `ValidationOutcome` 时保留后备分支。
7. 验证多次出现的值时，把验证结果转换为调用方拥有的 `ValidationReport`。本 crate 不会代替调用方调度或遍历这些验证任务。

准备工作发生在绑定期间，而不是每次验证调用时。`BoundValidator` 通过 `Arc` 持有预备验证器实例，因此可以克隆。

## 进阶用法：局部注册表与 inventory 注册表

使用 `ValidatorRegistry::from_registrations` 可以进行确定、显式的组合。该 API 始终可用，适合测试，也允许同一进程使用不同注册表。

进程级发现是可选能力，需要显式启用：

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

之后即可提交静态描述符并获得全局注册表。以下两段代码都来自 `examples/local_registry.rs`：

```rust
#[cfg(feature = "inventory")]
register_validator!(id = "text.non_blank.global", descriptor = &DESCRIPTOR);
```

```rust
#[cfg(feature = "inventory")]
{
    let global = ValidatorRegistry::try_global()?;
    assert!(global.get("text.non_blank.global").is_some());
}
```

使用以下命令编译并执行这些受 feature 控制的路径：

```bash
cargo run --example local_registry --all-features --locked
```

ID 重复会导致注册表构造失败，并返回 `ValidatorRegistryError::DuplicateId`。如果必须处理该失败，请优先使用 `try_global`；当链接得到的注册表无效时，`global` 会 panic。

## 依赖槽位

依赖签名是 `DependencySpec` 值的有序列表。名称让诊断信息更易理解，但不会把列表变成 map：位置、`InputType` 和可选性共同组成契约。如果调用方声明了正确的依赖，却采用了错误顺序，绑定仍会被拒绝。

下面的聚焦示例直接取自 `examples/local_registry.rs`：

```rust
fn assert_dependency_order_is_checked() {
    let declared = [DEPENDENCIES[1], DEPENDENCIES[0]];
    let error = DEPENDENCY_DESCRIPTOR
        .bind(ValidatorId::new("text.dependent"), 0, &[], &declared)
        .expect_err("swapped dependency slots must fail during binding");
    assert_eq!(error.kind(), BindErrorKind::DependencyOrderMismatch);
}
```

使用以下命令运行该源码：

```bash
cargo run --example local_registry --locked
```

执行时，`BoundValidationContext` 必须包含数量和形状均一致的值。只有可选槽位才能使用 `ValidationValue::Missing`。`new_with_paths` 可以为每个槽位关联结构化依赖路径，以便诊断。

## 错误与诊断

- 规则的领域错误属于直接类型化验证。适配器映射器将其转换为 `ViolationDraft`；已绑定验证器附加规则 ID，并在 `ValidationOutcome::Invalid` 中返回最终 `Violation` 值。
- `BindError` 报告配置失败，例如缺少规则、不支持的输入、格式错误的参数以及依赖声明不匹配。
- `ValidatorRegistryError` 报告注册表冻结期间遇到的重复 ID 或无效描述符。
- `ExecutionError` 报告基础设施和适配器失败，与业务值无效分开表示。
- `ValidationReport` 是由调用方选择的聚合结果。它可以限制存储的违规项和跳过项数量，并记录截断状态。

参数通过 `ArgumentReader` 解码。当前参数第一次被类型化读取时即被消耗，即使该读取因类型转换或范围转换而失败也是如此。再次读取会返回 `ParameterAlreadyConsumed`。以下代码直接取自 `examples/local_registry.rs`：

```rust
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
```

使用以下命令运行该源码：

```bash
cargo run --example local_registry --locked
```

公共诊断信息有意省略原始输入和原始参数值。`ExecutionError` 可以在内部保留源错误，但其 `Display` 和 `Debug` 输出不会暴露源错误文本。`ValidationPath::Display` 同样经过脱敏；受信任的展示代码必须显式选择调用 `ValidationPath::render`。

## 排障

- **`MissingRule`**：核对稳定 ID，并确认所选局部注册表包含该注册。使用全局注册表时，请启用 `inventory`，并确保注册规则的 crate 已链接。
- **`UnsupportedInput` 或 `InputTypeMismatch`**：绑定和调用时必须使用所选签名声明的确切 `InputType`。文本值和类型化值不会隐式转换。
- **依赖声明错误**：将提供的依赖切片与 `BoundValidator::dependency_specs` 对照；顺序很重要。
- **`UnknownParameter`**：消耗每个配置参数，再调用 `ArgumentReader::finish`。移除拼写错误或不受支持的名称。
- **`ParameterAlreadyConsumed`**：每个当前参数只解码一次，并将解码结果存入预备验证器。
- **`AdapterContractViolation`**：检查自定义 `PreparedValidator` 实现。无效结果必须至少包含一个草稿；跳过结果必须满足 API 文档规定的先决条件规则。

## 限制与最佳实践

- 本 crate 不发现字段、不遍历对象、不编译路径，也不调度多条规则。这些策略应由调用方实现。
- 保持验证器 ID 和违规项代码稳定。下游消费者应把这类变更视为协议变更。
- 除非确实需要进程级发现，否则优先使用局部注册表。
- 将依赖顺序视为类似 ABI 的契约。只有在调用方和实现完成协调后，才能追加或重排槽位。
- 普通 `Validator` 实现应使用项目提供的文本适配器或类型化适配器。自定义预备适配器必须遵守验证结果契约。
- 匹配公共非穷尽 enum 时保留通配分支，以便次要版本添加 variant。
- `ViolationParam` 中只能放适合展示的安全值；绝不能把被拒绝的输入加入错误消息或结构化参数。
- 接收不受信任或规模很大的验证任务集合时，使用 `ValidationReport::with_limits`。

## 延伸阅读

- [设计与不变量](design.zh_CN.md)
- [可运行的局部注册表示例](../examples/local_registry.rs)
- [API 文档](https://docs.rs/qubit-validator)
- [项目中文 README](../README.zh_CN.md)
- [English User Guide](user_guide.md)
- [English README](../README.md)
