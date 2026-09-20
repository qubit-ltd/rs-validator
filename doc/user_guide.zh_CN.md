# qubit-validator 用户手册

[English version](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-validator)

本文适用于 `qubit-validator` 0.1.0 和 Rust 1.94 及更高版本，面向需要复用验证规则、返回结构化结果，或按稳定标识符管理规则的应用与库开发者。

## 手册目标与问题边界

本 crate 负责验证边界，不负责遍历模型。调用方已经拿到待验证值时，可以用它执行类型化规则、为多次类型擦除调用预构造规则，或把违规结果收集到有上限的报告中。模型元数据和验证计划调度由其他组件负责。

## 概念模型

| 对象 | 用途 |
| --- | --- |
| `Validator<T, C>` | 针对借用值和不可变上下文的普通类型化规则。 |
| `PreparedValidator` | 可共享的类型擦除规则实例。 |
| `ValidatorSignature` 与 `ValidatorDescriptor` | 描述规则支持的输入形状、依赖槽位和准备函数。 |
| `ValidatorRegistry` | 按稳定 ID 管理注册项，并按输入形状绑定规则。 |
| `ValidationOutcome` | 表示通过、带 `Violation` 的失败，或显式跳过。 |
| `ValidationReport` | 有界收集违规项和跳过项。 |

这里有两条不同的错误边界：`BindError` 表示规则定义或调用无法准备；`ExecutionError` 表示准备好的规则无法在当前输入或依赖上执行。业务违规通过 `ValidationOutcome::Invalid` 返回，不应被当成执行故障。

## 贯穿场景：拒绝空白用户名

本场景的成功标准是：非空白用户名返回 `Valid`，只包含空白的输入返回代码为 `user.name.blank` 的结构化违规。

## 安装与最小配置

```toml
[dependencies]
qubit-validator = "0.1"
```

默认 feature 集为空。只有需要通过 `register_validator!` 做进程级注册时，才启用 `inventory`：

```toml
qubit-validator = { version = "0.1", features = ["inventory"] }
```

## 核心工作流

### 1. 编写并直接调用类型化规则

调用方已经知道具体输入类型时，实现 `Validator<T>`。默认上下文是 `()`；规则需要不可变类型化上下文时，实现 `Validator<T, C>`。

```rust
use qubit_validator::Validator;

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

assert!(NonBlank.validate("Ada", &()).is_ok());
assert!(NonBlank.validate("  ", &()).is_err());
```

### 2. 把规则适配到类型擦除边界

`prepare_text_validator` 接收 `Validator<str>`，并将领域错误映射为 `ViolationDraft`。准备好的规则在执行时借用输入，并可通过 `Arc` 共享。

```rust
use qubit_validator::BoundValidationContext;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;

let prepared = prepare_text_validator(NonBlank, |_| {
    ViolationDraft::new(ViolationCode::new("user.name.blank"))
});
let outcome = prepared
    .validate(ValidationValue::Text("  "), &BoundValidationContext::new(&[]))?;
assert!(matches!(outcome, PreparedOutcome::Invalid(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

如果输入是具体的 `T: 'static`，使用 `prepare_typed_validator`，并在执行时传入 `ValidationValue::Typed(&value)`。

### 3. 通过局部注册表绑定规则

需要按稳定 ID 查找规则时，声明 `ValidatorSignature`，将它放进静态的 `ValidatorDescriptor`，再用 `ValidatorId` 和 `RegistrationSource` 创建 `ValidatorRegistration`。使用 `ValidatorRegistry::from_registrations` 构造隔离的局部注册表。构造过程会按 ID 排序，并拒绝重复 ID 或无效 descriptor。

每个已配置的规则调用一次 `registry.bind(id, input_type, params, dependencies)`，然后复用得到的 `BoundValidator` 执行 `validate`。一个 descriptor 可以提供多个签名，但每种输入形状只能出现一次。

启用 `inventory` 后，可以使用 `register_validator!` 以及 `ValidatorRegistry::try_global()` / `global()` 做进程级发现。测试或租户需要独立规则集合时，优先使用局部注册表。

## 进阶用法

- 使用 `DependencySpec` 声明通过 `BoundValidationContext` 传入的有序依赖槽位。必需槽位不接受 `ValidationValue::Missing`；可选槽位接受该值，并可通过 `optional_typed` 读取。
- 使用 `NamedValidationArgument` 与 `ValidationArgument` 传递借用的、与领域无关的准备参数。
- 当执行器有意不运行规则时，返回带 `SkipReason::MissingOptional` 或 `SkipReason::FailedPrerequisite` 的 `PreparedOutcome::Skipped`。
- 使用 `ValidationPath::root().with_field("profile").with_index(0)` 构造嵌套位置；`ViolationDraft` 和 `Violation` 都可以携带路径及静态 `ViolationParam`。
- 需要限制保留的违规或跳过记录时，用 `ValidationReport::with_limits`；达到上限会将报告标记为截断。

## 错误与诊断

- `ValidatorRegistryError` 在构造注册表时报告重复 ID 和无效 descriptor。
- `BindError` 报告缺少规则、不支持或有歧义的签名、依赖声明不匹配以及准备失败。
- `ExecutionError` 报告输入形状不匹配、依赖缺失或类型错误、适配器契约违规以及规则执行故障。
- `ValidationOutcome::Invalid` 携带结构化 `Violation`；`ValidationOutcome::Skipped` 保留跳过原因和前置违规。

诊断时使用错误种类访问器以及其中记录的规则或依赖信息。绑定错误和执行错误都不能被当作验证通过。

## 排障

1. 如果绑定返回 `UnsupportedInput`，比较请求的 `InputType` 与 descriptor 中声明的签名。
2. 如果出现依赖错误，逐项核对 `DependencySpec` 的名称、顺序、输入形状和可选性。
3. 如果执行返回类型不匹配，确认 `ValidationValue` 和所有依赖槽位都使用了声明的准确形状。
4. 如果报告不是有效结果，检查 `violations()`、`skipped()` 和 `is_truncated()`；截断报告不保证穷尽所有结果。
5. 如果全局注册表初始化失败，检查是否存在重复注册 ID；使用 `try_global()` 可以保留结构化错误。

## 限制与最佳实践

`qubit-validator` 不遍历模型、不编译属性路径、不调度规则，也不内置领域规则。类型擦除边界有意只支持借用文本和基于准确 `TypeId` 的类型化值。准备好的 validator 必须满足 `Send + Sync`，因此其共享状态应保持不可变，或由实现自行同步。库不保证被截断的报告包含全部违规。

## 延伸阅读

- [中文 README](../README.zh_CN.md) · [English README](../README.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-validator)
