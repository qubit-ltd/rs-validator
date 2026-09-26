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
let validator = registry.bind("text.non_blank", InputType::Text, &[])?;
let context = BoundValidationContext::new(&[]);

let accepted = validator.validate(ValidationValue::Text("Ada"), &context)?;
assert_eq!(accepted, ValidationOutcome::valid());

let rejected = validator.validate(ValidationValue::Text("  "), &context)?;
let mut report = ValidationReport::new();
assert!(report.record_outcome(0, ValidationPath::root(), rejected)?.complete());
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

当规则需要将目标值与已选择的依赖值比较时，使用 context-aware adapter。`validate` 要求依赖按签名声明顺序传入，并检查其形状以及必需/可选属性。直接调用者可使用 `validate_named` 按名称绑定每个值，避免同类型槽位被静默互换；该入口会分配临时重排缓冲。

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

当规则需要返回多条违规，或要区分数据无效与执行故障时，使用闭包适配器。
闭包通过 `PreparedOutcome` 返回验证结果，通过 `ExecutionError` 返回执行错误：

```rust
let prepared = prepare_text_with_context(|value, context| {
    let expected = context.text(0)?;
    if value == expected {
        Ok(PreparedOutcome::Valid)
    } else {
        Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(
            ViolationCode::new("text.dependency_mismatch"),
        )]))
    }
});
```

适配器会检查目标值为文本。`BoundValidator` 在调用闭包前检查依赖数量、类型、
可选性和路径。不要把原始输入写入违规项或公开错误格式。

## 汇总验证结果与先决条件

`record_outcome` 要求成功调用的 occurrence 非递减；同一位置可重复记录多个结果。传入较小位置会返回 `ValidationOutcomeError::OutOfOrderOccurrence`，且不修改报告。结果按调用顺序追加，以保持已签发 `FailureId` 的索引稳定。无效结果至少包含一个违规项。先决条件失败的跳过项通过不透明 `FailureId` 引用同一报告中已保留的违规项；报告会先校验引用再修改状态。引用不占用 `max_violations`，原始违规项只计数一次；`max_skipped` 限制跳过 occurrence 数。`record_outcome` 返回 `RecordedOutcome`：`complete()` 表示本次结果是否完整写入，`failure_ids()` 返回本次保留的原始失败 ID。`ValidationReport::failures()` 只遍历原始违规项，`failure(id)` 可解析对应违规项。拥有型执行原因仅通过显式可信入口 `trusted_source()` 读取，普通错误格式化和 `Error::source()` 不暴露底层文本。
```rust
let rule_id = ValidatorId::new("text.required");
let earlier = Violation::new(rule_id, ViolationCode::new("text.blank"));
let mut report = ValidationReport::new();
let original = report.record_outcome(
    0,
    ValidationPath::root().with_field("password"),
    ValidationOutcome::invalid(vec![earlier])?,
)?;
assert!(original.complete());
let failure_id = original.failure_ids()[0];
let skipped = report.record_outcome(
    1,
    ValidationPath::root().with_field("confirmation"),
    ValidationOutcome::failed_prerequisite(vec![failure_id])?,
)?;
assert!(skipped.complete());
assert_eq!(report.violations().len(), 1);
assert_eq!(report.failure(failure_id).unwrap().path(), &ValidationPath::root().with_field("password"));
assert_eq!(report.skipped()[0].path(), &ValidationPath::root().with_field("confirmation"));
assert_eq!(report.skipped()[0].prerequisites(), &[failure_id]);
assert_eq!(report.failure_count(), 1);
assert_eq!(report.failures().count(), report.failure_count());
```

`RecordedOutcome::complete()` 表示本次结果是否完整写入，不代表验证是否通过。容量不足时返回不完整回执并标记报告已截断；结果形状不合法时返回 `ValidationOutcomeError`，报告保持不变。`max_violations` 只约束保留的原始违规项。失败容量耗尽时，不会留下证据列表为空的先决条件失败跳过记录；若跳过记录被 `max_skipped` 拒绝，其先决条件证据也不占失败容量。

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
- `ExecutionError` 表示输入/依赖形状错误、适配器契约问题或外部执行失败。底层拥有型原因只可通过显式可信入口 `trusted_source()` 读取；普通 `Display`、`Debug` 和 `Error::source()` 不会暴露它。
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
| `!recorded.complete()` | 报告容量拒收了部分或全部结果；检查 `is_truncated()` 并按实际负载配置上限。 |

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
