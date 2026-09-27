# qubit-validator 中文用户手册

[中文 README](../README.zh_CN.md) · [English user guide](user_guide.md) · [API 文档](https://docs.rs/qubit-validator)

本文适用于 `qubit-validator` 0.1.x，最低 Rust 版本为 1.94。面向需要在 Rust 应用中验证输入、按配置选择规则并返回结构化问题的开发者。读完[接入资料修改流程](#接入资料修改流程)和[检查验证结果](#检查验证结果)，即可完成一次规则注册、执行和结果汇总；依赖字段、参数、报告容量与全局发现可在需要时再查。

## 它解决什么问题

以用户修改资料为例。资料模块收到 `display_name`，需要拒绝全空白名称，并把问题标到相应字段。起初直接调用一个 Rust 规则就够了；当不同入口按配置选择规则时，应用还需要稳定的规则 ID、可在启动阶段检查的配置，以及不会把用户输入写进日志或诊断的结果。

`qubit-validator` 提供两层用法：`Validator<T, C>` 让规则保持普通的类型化 Rust 代码；注册表和适配器让应用按 ID 绑定规则，再用 `ValidationReport` 收集结构化违规项。资料模块仍负责选择字段、调用规则、决定是否写入数据以及如何对外展示错误。本库不会自动遍历资料对象或保存数据。

## 从哪里开始

1. 按[接入资料修改流程](#接入资料修改流程)定义空白名称规则，在启动时注册、绑定，再在请求中复用。
2. 按[检查验证结果](#检查验证结果)区分验证通过、业务输入不符合规则和执行失败，确认字段路径与违规代码。
3. 需要跨字段比较时看[让规则读取另一个字段](#让规则读取另一个字段)；需要配置参数、跳过后续规则、有界报告或进程级注册时，再查后续章节。

仓库的 [`local_registry.rs`](../examples/local_registry.rs) 是可运行的入门练习，覆盖局部注册表、依赖名称匹配和参数读取。下文将相同的公开 API 放入资料模块的调用流程；其中资料存储和 HTTP 响应属于应用代码，由读者自行实现。

## 接入资料修改流程

在应用的 `Cargo.toml` 中加入：

```toml
[dependencies]
qubit-validator = "0.1"
```

默认 feature 已支持直接调用规则、适配器和局部注册表。先把“空白名称无效”写成类型化规则。`BlankText` 是资料模块定义的领域错误；直接调用 `NonBlank.validate(name, &())` 时，调用方可以按自己的业务逻辑处理它。

```rust
use std::{error::Error, fmt};
use qubit_validator::Validator;

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

    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
    }
}
```

如果资料模块要按配置中的 `text.non_blank` 查找规则，就在启动前定义它的准备函数和签名。准备函数负责读取配置参数，并把领域错误映射为稳定的违规代码 `text.blank`；这里没有参数，因此 `finish()` 会拒绝任何多余参数。映射器不要放入原始名称或领域错误文本。

`ViolationParam` 是受限的诊断值词汇，不是值安全或可信来源的证明。来自规则配置的 `Unsigned(minimum)` 可以使用；来自被拒绝输入的 `Unsigned(rejected_number)` 不可以使用。`Token` 的 `'static` 生命周期不能证明它来自可信声明。调用方不得放入被拒绝输入或由其派生的敏感值。

```rust
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, InputType, NamedValidationArgument,
    PreparedValidator, ValidatorDescriptor, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

fn prepare_non_blank(
    params: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    ArgumentReader::new(params)?.finish()?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_non_blank),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
```

启动时将规则登记到局部注册表，再按 ID 和输入类型绑定。`RegistrationSource` 留下注册位置，便于排查重复 ID；绑定时会选择签名并运行一次准备函数。应用可将得到的 `BoundValidator` 放入资料服务并供请求复用。规则执行是同步的，目标值只在本次调用期间借用。

```rust
use qubit_validator::{
    BoundValidator, RegistrationSource, ValidatorId,
    ValidatorRegistration, ValidatorRegistry,
};

fn bind_display_name_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    Ok(registry.bind("text.non_blank", InputType::Text, &[])?)
}
```

下面是请求处理阶段的验证部分。`display_name` 来自请求，`validator` 是启动时注入的已绑定规则。应用应先检查报告，再决定是否把资料交给自己的存储层；`qubit-validator` 不替应用提交更新。

```rust
use qubit_validator::{
    BoundValidationContext, ValidationPath, ValidationReport, ValidationValue,
};

fn check_display_name(
    validator: &BoundValidator,
    display_name: &str,
) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let context = BoundValidationContext::new(&[]);
    let outcome = validator.validate(ValidationValue::Text(display_name), &context)?;
    let mut report = ValidationReport::new();
    let recorded = report.record_outcome(
        0,
        ValidationPath::root().with_field("display_name"),
        outcome,
    )?;
    assert!(recorded.complete()); // 此处使用无上限报告
    Ok(report)
}
```

这里的 `0` 是这次检查在调用方计划中的 occurrence 编号，`display_name` 是资料字段路径。应用调用 `check_display_name(&validator, "Ada")` 会得到无违规报告；传入 `"   "` 会得到一条位于 `display_name`、代码为 `text.blank` 的违规项。可用仓库示例核对同一条规则的绑定和输出：

```bash
cargo run --example local_registry --locked
```

在 `rs-validator` 仓库目录中运行此命令。示例成功时退出码为 0，不打印业务响应；它用断言检查 `"Ada"` 有效、空白文本生成 `text.blank`。

### 这些对象各负责什么

| 对象 | 在资料流程中的作用 |
| --- | --- |
| `Validator<str>` | 实现空白名称的业务判断，可直接调用。 |
| `ValidatorSignature` 与 `ValidatorDescriptor` | 声明此规则接受文本、没有依赖，以及如何准备实例。 |
| `ValidatorRegistry` | 保存以稳定 ID 命名的规则定义；局部注册表可由应用显式构造。 |
| `BoundValidator` | 保存选定签名和预备实例，检查调用时的输入形状并附加规则 ID。 |
| `ValidationOutcome` | 表示一次执行已通过、违反规则或由调用方跳过。 |
| `ValidationReport` | 按字段和 occurrence 收集多次执行的结果。 |

## 检查验证结果

业务入口应分别处理三个阶段：

1. **启动或配置阶段**：`from_registrations` 可能因重复 ID 或无效描述符返回 `ValidatorRegistryError`；`bind` 可能因规则不存在、输入签名不匹配或参数不合法返回 `BindError`。应在开放请求前处理这些配置错误。
2. **请求执行阶段**：`validate` 返回 `Ok(ValidationOutcome::Valid)` 表示这条规则通过；返回 `Ok(ValidationOutcome::Invalid(...))` 表示规则已经执行、输入不符合要求。`Err(ExecutionError)` 表示输入或依赖形状不对、适配器契约错误或规则执行故障，不能当成普通字段错误。
3. **报告汇总阶段**：`record_outcome` 将违规项附到调用方给出的字段路径，返回 `RecordedOutcome`。`complete()` 表示本次结果完整写入，**不表示验证通过**；是否可接受要看 `report.is_valid()`，并在设置容量上限时检查 `is_truncated()`。

例如，资料服务可以把验证结果与存储结果分开返回。以下 `ProfileStore` 由应用实现，成功返回表示其存储操作已完成；`UpdateProfileResult` 也是应用的响应模型：

```rust
trait ProfileStore {
    fn save_display_name(
        &self,
        user_id: &str,
        display_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

enum UpdateProfileResult {
    Updated,
    Rejected(ValidationReport),
}

fn update_profile(
    store: &dyn ProfileStore,
    validator: &BoundValidator,
    user_id: &str,
    incoming_display_name: &str,
) -> Result<UpdateProfileResult, Box<dyn std::error::Error>> {
    let report = check_display_name(validator, incoming_display_name)?;
    if !report.is_valid() {
        return Ok(UpdateProfileResult::Rejected(report));
    }
    store.save_display_name(user_id, incoming_display_name)?;
    Ok(UpdateProfileResult::Updated)
}
```

资料存储实现与 HTTP handler 由应用提供。`"Ada"` 通过后会调用存储，空白名称会返回 `Rejected(report)` 且不会调用存储；存储失败则通过 `Err` 返回，不会被伪装成输入无效。handler 可遍历 `report.violations()`，将 `violation.code()` 与受控字段路径映射为 API 错误，避免在消息或日志中写入 `incoming_display_name`。`ValidationPath` 保留结构化字段信息，但默认 `Display`/`Debug` 不显示字段名及 map 位置。只有受信任且允许披露路径的展示层才调用 `render()`。稳定违规代码可由调用方映射为面向用户的文案；本库不负责消息本地化。

## 让规则读取另一个字段

资料表单可能要求“确认邮箱”与“邮箱”相同。此时规则除了目标值，还要读取一个由调用方选出的依赖值。签名中的依赖槽位规定名称、类型和可选性；`BoundValidator` 在进入规则前核验依赖数量与形状。

以下规则只比较文本，不把两个邮箱值写入违规项。`DEPENDENCIES` 同时交给适配器和签名；`DependencyMismatch` 是应用定义并实现了 `std::error::Error` 的领域错误，定义方式可参照前面的 `BlankText`。

```rust
use qubit_validator::{
    BoundValidationContext, DependencySpec, InputType, Validator,
    ViolationCode, ViolationDraft, prepare_contextual_text_validator,
};

static DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("email", InputType::Text, false),
];

struct MatchesEmail;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesEmail {
    type Error = DependencyMismatch;

    fn validate(
        &self,
        confirmation: &str,
        context: &BoundValidationContext<'a>,
    ) -> Result<(), Self::Error> {
        let email = context.text(0).map_err(|_| DependencyMismatch)?;
        if confirmation == email { Ok(()) } else { Err(DependencyMismatch) }
    }
}

let prepared = prepare_contextual_text_validator(DEPENDENCIES, MatchesEmail, |_| {
    ViolationDraft::new(ViolationCode::new("email.confirmation_mismatch"))
});
```

末尾的适配器创建语句应放在准备函数中；应用仍需按第一节的做法声明 `ValidatorSignature`、注册并绑定此规则。绑定后，直接调用方可以按名称传依赖，避免多个同类型槽位意外交换：

```rust
use qubit_validator::{NamedValidationDependency, ValidationValue};

let dependencies = [NamedValidationDependency::new(
    "email", ValidationValue::Text(email),
)];
let outcome = validator.validate_named(
    ValidationValue::Text(confirmation),
    &dependencies,
)?;
```

这里的 `validator` 是已绑定的确认邮箱规则，`email` 和 `confirmation` 由请求提供。相同时返回 `Valid`，不同时返回带 `email.confirmation_mismatch` 的 `Invalid`；缺少 `email` 绑定或传入错误类型则返回 `ExecutionError`。`validate_named` 会按签名名称重排并分配临时缓冲。若上游已经核验依赖顺序，可改用 `validate` 和 `BoundValidationContext::new(&values)`；这时所有值必须严格按签名槽位顺序放置。

可选依赖也要占有对应槽位，缺失时显式传入 `ValidationValue::Missing`。规则可用 `context.optional_typed::<T>(index)` 读取可选类型值；类型错误不会被当作缺失。文本规则使用 `prepare_contextual_text_validator`，类型化目标值可使用 `prepare_contextual_typed_validator::<T, _, _, _>`。无需上下文的规则沿用简单适配器。

## 给规则传配置参数

如果资料策略规定名称最大长度，配置层可在绑定时传入命名参数，而不必在每次请求中解码。准备函数用 `ArgumentReader` 一次性读取支持的参数并构造预备规则，例如：

```rust
use qubit_validator::{ArgumentReader, BindError, NamedValidationArgument};

fn read_max_length(params: &[NamedValidationArgument<'_>]) -> Result<u32, BindError> {
    let mut reader = ArgumentReader::new(params)?;
    let maximum = reader.required_u32("maximum")?;
    reader.finish()?;
    Ok(maximum)
}
```

上面只展示参数解码；调用方仍需把 `maximum` 保存到自己的预备规则，并在规则执行时使用。传入 `NamedValidationArgument::new("maximum", ValidationArgument::Unsigned(40))` 可得到 `40`；缺少、类型错误、越界或出现未消费参数会在绑定阶段返回 `BindError`。每个已提供参数首次按类型读取后即被消费，即使转换失败也一样；再次读取会返回 `ParameterAlreadyConsumed`。配置修正后应重新绑定，不要在同一个 reader 上重试。

## 多条规则如何汇总与跳过

一个资料字段可能有多条规则。调用方按非递减 occurrence 编号调用 `record_outcome`；同一编号可记录多条结果。传入更小编号会返回 `ValidationOutcomeError::OutOfOrderOccurrence`，且报告不变。一次验证结果中的 `Invalid` 至少要有一条违规；结果 enum 非穷尽，匹配时应保留兜底分支。

如果后续检查以先前失败为前提，调用方可以把先前 `record_outcome` 返回的 `failure_ids()` 交给 `ValidationOutcome::failed_prerequisite`，只记录一次原始失败：

```rust
use qubit_validator::{
    ValidationOutcome, ValidationPath, ValidationReport,
    ValidatorId, Violation, ViolationCode,
};

let mut report = ValidationReport::new();
let first = report.record_outcome(
    0,
    ValidationPath::root().with_field("password"),
    ValidationOutcome::invalid(vec![Violation::new(
        ValidatorId::new("text.required"),
        ViolationCode::new("text.blank"),
    )])?,
)?;
let later = report.record_outcome(
    1,
    ValidationPath::root().with_field("confirmation"),
    ValidationOutcome::failed_prerequisite(first.failure_ids().to_vec())?,
)?;
assert!(first.complete() && later.complete());
assert_eq!(report.failure_count(), 1);
assert_eq!(report.skipped().len(), 1);
```

跳过策略由调用方决定，本库不会自动调度这些规则。`FailureId` 只可引用同一份报告中已保留的违规项；跨报告或未保留的 ID 会使 `record_outcome` 返回错误。可选目标缺失时可记录 `ValidationOutcome::missing_optional()`；它与先决条件失败是不同原因。调用方可用 `report.failures()` 只遍历原始违规，并用 `report.failure(id)` 找到跳过项引用的失败。

当请求可能触发大量规则时，可限制保留数量：

```rust
use qubit_validator::{ValidationLimits, ValidationReport};

let report = ValidationReport::with_limits(ValidationLimits {
    max_violations: Some(20),
    max_skipped: Some(20),
});
```

`max_violations` 计入保留的原始违规项，跳过项对失败 ID 的引用不重复占用该额度；`max_skipped` 限制跳过记录数量。容量不足时 `record_outcome` 可能返回 `!complete()`，报告也会标记 `is_truncated()`。这时不能把未收集到的错误解释为“其余字段都通过”。如果调用方自行提前停止执行，应调用 `mark_truncated()` 表明报告并不完整。

## 何时使用其他适配器或注册方式

规则需要一次返回多条违规，或要把外部执行失败与业务输入无效分开时，可用 `prepare_text_with_context` / `prepare_typed_with_context`：闭包返回 `PreparedOutcome` 表示已执行的验证结果，返回 `ExecutionError` 表示执行故障。已有带上下文的领域规则则可用 `prepare_text_domain_rule` / `prepare_typed_domain_rule`，将领域错误映射为一条或多条安全违规草稿；基础设施错误继续以 `ExecutionError` 返回。草稿列表不能是空的，否则会成为适配器契约错误。

默认优先使用显式的 `ValidatorRegistry::from_registrations`：一个进程可以构造多组注册表，也便于测试和控制启动配置。只有确需从已链接 crate 自动发现规则，才启用 `inventory`：

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

然后在规则定义处使用 `register_validator!`，启动时调用 `ValidatorRegistry::try_global()` 处理重复 ID 或无效描述符。`global()` 遇到这些问题会 panic。`inventory` 改变规则发现方式，不改变绑定和执行契约。仓库示例包含启用该 feature 时的注册路径。

若已有一个无依赖、精确接受 `T` 的 `Arc<dyn PreparedValidator>`，可使用 `BoundValidator::try_from_prepared::<T>` 直接绑定；输入类型或依赖不符时返回 `BindError`。

## 按症状排查

| 现象 | 检查位置与处理 |
| --- | --- |
| `MissingRule` | 检查绑定所用注册表是否包含该 ID；使用全局注册时确认启用 `inventory` 且注册所在 crate 已链接。 |
| `UnsupportedInput`、`InputTypeMismatch` | 检查签名的 `InputType` 与传入的 `ValidationValue`；文本不会自动转成类型化值。 |
| 参数报 `UnknownParameter`、`ParameterAlreadyConsumed` | 在准备函数中按名称各读取一次，最后调用 `finish()`；修正配置后重新绑定。 |
| 依赖缺失或类型不符 | 核对槽位名称、顺序、类型和可选性；可选缺失必须显式使用 `Missing`。 |
| `AdapterContractViolation` | 检查自定义 `PreparedValidator` 的输入/依赖声明和返回的草稿，尤其是空的无效结果。 |
| 报告 `!complete()` 或 `is_truncated()` | 检查 `ValidationLimits` 和调用方是否提前停止；不要将部分结果当作完整验证。 |
| 路径在日志中看不到 | `ValidationPath` 的默认格式会隐藏敏感位置；仅在受信任展示层按需调用 `render()`。 |

`ExecutionError` 的普通 `Display`、`Debug` 和 `Error::source()` 不暴露底层拥有型原因；只有明确可信的诊断路径才使用 `trusted_source()`。错误和违规参数不应包含被拒绝的原始输入。违规代码与规则 ID 可能被其他模块作为稳定契约使用，变更时应同步调用方。

本库同步执行，预备验证器需满足 `Send + Sync`；它不创建线程，也不要求异步运行时。对象遍历、规则调度、请求响应、数据持久化和消息本地化由应用负责。

## 延伸阅读

- [完整局部注册表示例](../examples/local_registry.rs)
- [设计与不变量](design.zh_CN.md)
- [中文 README](../README.zh_CN.md)
- [API 文档](https://docs.rs/qubit-validator)
- [English user guide](user_guide.md)：英文手册为独立版本，内容可能未与本文逐节同步。
