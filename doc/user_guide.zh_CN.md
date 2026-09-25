# qubit-validator 用户指南

[English](user_guide.md)

适用版本：`qubit-validator` 0.1.x · 最低 Rust 版本：1.94

## 手册目标与读者

本指南面向 Rust 库和应用的开发者：他们既要直接调用类型化规则，也需要按稳定 ID 配置和查找规则。本文介绍公共 API，不要求项目使用特定模型框架、代码生成器或调度器。

## 概念模型

| 概念 | 含义 |
| --- | --- |
| `Validator<T, C>` | 带具体输入和上下文类型的规则，可直接调用并返回领域错误。 |
| `PreparedValidator` | 配置解码后创建的可复用类型擦除实例。 |
| `BoundValidator` | 将预备实例与稳定规则 ID、选定签名绑定起来；执行前检查输入和依赖形状。 |
| `ValidationOutcome` | 一次验证完成后的结果：有效、无效或跳过。输入无效不等于执行错误。 |
| `ExecutionError` | 输入或依赖类型不符、适配器契约错误、外部执行失败等，以 `Err` 返回；不保留底层 source。 |
| `ValidationReport` | 由调用方拥有的违规项和跳过记录集合，可设置数量上限。 |

适配器负责把规则的领域错误映射为安全的 `ViolationDraft`。绑定后的验证器再附加稳定规则 ID，返回最终 `Violation`。

## 场景：检查配置的显示名称

本指南以 `NonBlank` 规则检查显示名称。规则用 `text.non_blank` 注册，领域错误映射为 `text.blank`，随后绑定一次并重复使用，分别检查有效和无效输入。完整的可运行代码在 [`examples/local_registry.rs`](../examples/local_registry.rs)，运行命令：

```bash
cargo run --example local_registry --locked
```

## 安装与最小配置

直接验证和局部注册表不需要启用可选 feature：

```toml
[dependencies]
qubit-validator = "0.1"
```

默认配置就支持局部注册表。准备函数解码参数并返回 `Arc<dyn PreparedValidator>`；静态 `ValidatorSignature` 与 `ValidatorDescriptor` 描述规则的输入和依赖。完整定义见上方链接的示例。调用点可以这样绑定和执行：

```rust
let registry = ValidatorRegistry::from_registrations([registration])?;
let validator = registry.bind("text.non_blank", InputType::Text, &[], &[])?;
let context = BoundValidationContext::new(&[]);

let accepted = validator.validate(ValidationValue::Text("Ada"), &context)?;
assert_eq!(accepted, ValidationOutcome::valid());

let rejected = validator.validate(ValidationValue::Text("  "), &context)?;
let mut report = ValidationReport::new();
assert!(report.record_outcome(0, ValidationPath::root(), rejected)?);
assert!(!report.is_valid());
```

## 核心工作流

1. 为规则实现 `Validator<T, C>`。如果不需要运行时选择，直接调用它即可。
2. 没有依赖槽位时，使用 `prepare_text_validator` 或 `prepare_typed_validator`。映射器把领域错误转换为稳定违规代码和安全参数。
3. 用 `ValidatorSignature` 声明输入形状及有序依赖，再将签名放入 `ValidatorDescriptor`，并关联 `ValidatorId` 和 `RegistrationSource`。
4. 创建局部 `ValidatorRegistry`，绑定规则并复用生成的 `BoundValidator`。准备函数只在绑定时执行一次。
5. 每次调用传入借用的 `ValidationValue` 和 `BoundValidationContext`。匹配非穷尽公共 enum 时保留兜底分支。
6. 需要汇总多次结果时，将每个 `ValidationOutcome` 交给 `ValidationReport::record_outcome`。本 crate 不负责遍历对象或调度规则组。

对于无依赖的已准备规则，可使用 `BoundValidator::from_prepared<T>`。它跳过注册表查找和参数准备，但每次调用仍会检查输入类型和依赖数量。

## 进阶用法：读取依赖的适配器

当规则需要将目标值与已选择的依赖值比较时，使用 context-aware adapter。签名声明依赖槽位，`BoundValidator` 会先检查顺序、输入形状以及必需/可选属性，再调用类型化验证器。

假设 `DependencyMismatch` 是规则自己定义并实现 `std::error::Error` 的领域错误：

```rust
struct MatchesExpected;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesExpected {
    type Error = DependencyMismatch;

    fn validate(
        &self,
        value: &str,
        context: &BoundValidationContext<'a>,
    ) -> Result<(), Self::Error> {
        let expected = context.text(0).map_err(|_| DependencyMismatch)?;
        if value == expected { Ok(()) } else { Err(DependencyMismatch) }
    }
}

let prepared = prepare_contextual_text_validator(MatchesExpected, |_| {
    ViolationDraft::new(ViolationCode::new("text.dependency_mismatch"))
});
```

映射器不应把两个文本值写入错误或违规参数。可选类型槽位可通过 `context.optional_typed::<T>(index)` 读取；只有显式的 `ValidationValue::Missing` 会得到 `None`，错误类型仍会作为执行错误处理。

文本规则使用 `prepare_contextual_text_validator`，类型化规则使用 `prepare_contextual_typed_validator::<T, _, _, _>`。不需要读取上下文时，原有简单 adapter 仍是更直接的选择。

## 汇总验证结果与先决条件

`record_outcome` 负责统一出现顺序和报告容量。无效结果至少要有一个违规项；因先决条件失败而跳过时，也必须保留至少一个前置违规项。后者嵌套在 skipped entry 中，不会出现在报告顶层违规项列表。出现路径只会为无效结果中的违规项相对路径添加一次前缀；根路径表示出现位置本身。先决条件证据保留指向原始失败位置的绝对路径。跳过结果的出现路径用于定位被跳过的目标。规则准备层只返回 `Valid` 或 `Invalid`，跳过结果由调用方构造。`with_field` 使用静态声明名称，运行时 map 位置使用 `MapEntry`。`report.failures()` 先遍历顶层违规项，再按 skipped entry 顺序遍历先决条件证据；它保留重复项，不承诺全局出现顺序，迭代数量等于 `failure_count()`。

```rust
let rule_id = ValidatorId::new("text.required");
let earlier = Violation::new(rule_id, ViolationCode::new("text.blank"));
let mut report = ValidationReport::new();
assert!(report.record_outcome(
    0,
    ValidationPath::root().with_field("password"),
    ValidationOutcome::invalid(vec![earlier])?,
)?);
let earlier = report.violations()[0].clone();
assert!(report.record_outcome(
    1,
    ValidationPath::root().with_field("confirmation"),
    ValidationOutcome::failed_prerequisite(vec![earlier])?,
)?);
assert_eq!(report.violations().len(), 1);
assert_eq!(report.violations()[0].path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.skipped()[0].path(), &ValidationPath::root().with_field("confirmation"));
assert_eq!(report.skipped()[0].prerequisites()[0].path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.failure_count(), 2);
assert_eq!(report.failures().count(), report.failure_count());
```

返回的 `bool` 表示本次结果是否完整放入报告，不表示验证是否通过。若容量不足，API 返回 `Ok(false)` 并标记报告已截断；结果形状不合法时返回 `ValidationOutcomeError`，报告保持不变。`max_violations` 约束顶层违规项与先决条件证据的保留总数。失败容量耗尽时，不会留下证据列表为空的先决条件失败跳过记录；若跳过记录被 `max_skipped` 拒绝，其先决条件证据也不占失败容量。

## 局部注册表与 inventory

`ValidatorRegistry::from_registrations` 构造显式局部注册表，行为确定，适合测试，也允许一个进程使用多组规则。只有确实需要链接 crate 自动注册时，才启用 `inventory`：

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

使用 `ValidatorRegistry::try_global` 可以处理重复 ID 和无效描述符；`global` 遇到这两类注册错误都会 panic。该 feature 只改变发现方式，不改变规则绑定和执行契约。

## 错误与诊断

- 类型化规则返回领域错误，适配器映射器将其转换为 `ViolationDraft`。
- `BindError` 表示配置问题，例如规则缺失、输入不支持、参数格式不符或依赖声明不匹配。
- `ValidatorRegistryError` 报告重复 ID 和无效描述符。
- `ExecutionError` 表示输入/依赖形状错误、适配器契约问题或外部执行失败。`Display`、`Debug` 和标准错误链都不会保留或暴露底层 source error。
- `ValidationOutcome::Invalid` 是验证已完成的结果，不是 `ExecutionError`。

`ArgumentReader` 在参数第一次被类型化读取时就会消费它，即使类型或范围转换失败也一样。再次读取会返回 `ParameterAlreadyConsumed`。读取完支持的参数后调用 `finish`，以拒绝未消费的名称。

`ValidationPath` 始终以结构化形式保存。默认 `Display` 和 `Debug` 不显示字段名及 map 位置；只有受信任的展示层在适合披露时才显式调用 `ValidationPath::render`。不要把原始输入放入错误或 `ViolationParam`。

## 排障

| 现象 | 检查方法 |
| --- | --- |
| `MissingRule` | 确认选用的局部注册表包含该 ID。使用全局注册时启用 `inventory`，并确保注册所在 crate 已链接。 |
| `UnsupportedInput` 或 `InputTypeMismatch` | 使用签名声明的确切 `InputType`；不会自动把文本转换为类型化值。 |
| 依赖声明错误 | 对照签名检查顺序、输入形状和可选性。 |
| 执行时缺少依赖 | 检查必需槽位是否提供值；可选缺失必须使用 `ValidationValue::Missing`。 |
| `UnknownParameter` | 读取所有支持的参数后调用 `ArgumentReader::finish`。 |
| `ParameterAlreadyConsumed` | 每个参数只解码一次，并把结果保存在预备验证器中。 |
| `AdapterContractViolation` | 检查自定义 `PreparedValidator`。无效结果必须包含违规项；`PreparedOutcome` 只有有效和违规两种状态。 |
| `record_outcome` 返回 `Ok(false)` | 报告容量拒收了部分或全部结果；检查 `is_truncated()` 并按实际负载配置上限。 |

## 限制与最佳实践

- 本 crate 不发现字段、不遍历对象、不编译模型路径，也不调度多条规则；这些逻辑留给调用方。
- 稳定规则 ID 和违规代码会被下游使用，变更时要协调消费者。
- 没有进程级发现需求时优先使用局部注册表。
- 把依赖顺序视为类似 ABI 的契约；修改槽位前要同步所有调用方和规则实现。
- 面向不受信任或大型任务集时，使用 `ValidationReport::with_limits`。`max_violations` 共同约束保留的顶层违规项和先决条件证据，`max_skipped` 约束跳过条目数。超出容量的证据不会保留，报告会标记为已截断。
- 验证是同步的，输入只在调用期间借用。预备验证器必须满足 `Send + Sync`；本 crate 不创建线程，也不要求异步运行时。

## 延伸阅读

- [设计与不变量](design.zh_CN.md)
- [完整局部注册表示例](../examples/local_registry.rs)
- [API 文档](https://docs.rs/qubit-validator)
- [English README](../README.md)
- [中文 README](../README.zh_CN.md)
- [English User Guide](user_guide.md)
