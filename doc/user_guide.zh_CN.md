# Qubit Validator 用户手册

[中文 README](../README.zh_CN.md) · [English user guide](user_guide.md) · [API 文档](https://docs.rs/qubit-validator)

本文适用于 `qubit-validator` 0.1.x，最低 Rust 版本为 1.94。它面向需要在 Rust 应用中校验输入、按配置选择规则的开发者。读完[接入资料修改流程](#接入资料修改流程)和[保存前检查结果](#保存前检查结果)，就能完成基础接入；跨字段比较、参数、报告限额和其他注册方式可按需查阅。

## 目录

- [它解决什么问题](#它解决什么问题)
- [接入资料修改流程](#接入资料修改流程)
- [保存前检查结果](#保存前检查结果)
- [比较资料中的两个字段](#比较资料中的两个字段)
- [在启动时配置规则](#在启动时配置规则)
- [汇总多项检查并处理容量上限](#汇总多项检查并处理容量上限)
- [选择注册方式和适配器](#选择注册方式和适配器)
- [选择稳定的规则 ID 和违规代码](#选择稳定的规则-id-和违规代码)
- [排查故障并守住边界](#排查故障并守住边界)

## 它解决什么问题

用户修改资料时，资料模块收到 `display_name`。它必须拒绝全空白名称，把问题指出在对应字段，却不能把用户提交的名称放进诊断内容。最初直接调用一个 Rust 规则就够了；当多个入口根据配置选择规则时，还需要稳定的规则 ID、在启动阶段发现错误配置的办法，以及能转换为应用响应的结构化结果。

`qubit-validator` 将这些步骤分开：`Validator<T, C>` 是带类型的普通规则；描述符和注册表按 ID 绑定规则；已绑定的规则在每次调用时检查输入形状；`ValidationReport` 把结果附到应用指定的字段路径。资料模块仍负责选字段、安排检查顺序、决定是否保存和生成用户文案。本库不会自动遍历资料对象，也不负责写入数据库。

仓库的[局部注册表示例](../examples/local_registry.rs)可用 `cargo run --example local_registry --locked` 运行。它验证相同的 `text.non_blank` 规则和绑定流程，还覆盖命名依赖与参数读取；它没有实现资料存储或汇总报告。下文展示资料模块与这些边界如何衔接。若要根据字段注解自动校验整个对象，可先看[模型注解集成](../README.zh_CN.md#模型注解集成)中的本地依赖配置，再按[模型校验手册](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.zh_CN.md#实战场景续校验一个用户实例)完成模型级校验流程。

## 接入资料修改流程

先在应用的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
qubit-validator = "0.1"
```

默认功能已足够完成此流程。先在资料模块定义“全空白名称无效”规则。直接调用这条规则的代码可以处理它自己的领域错误；按配置执行时，再把领域错误转换为稳定违规代码。

<!-- validator-guide:rules -->
```rust
// src/profile/rules.rs
use std::{error::Error, fmt, sync::Arc};
use qubit_validator::{
    ArgumentReader, BindError, InputType,
    PreparedValidator, Validator, ValidatorDescriptor, ValidatorSignature,
    ViolationCode, ViolationDraft, prepare_text_validator,
};

#[derive(Debug)]
pub struct BlankText;
impl fmt::Display for BlankText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("text must not be blank")
    }
}
impl Error for BlankText {}

pub struct NonBlank;
impl Validator<str> for NonBlank {
    type Error = BlankText;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() { Err(BlankText) } else { Ok(()) }
    }
}

fn prepare(_reader: &mut ArgumentReader<'_>)
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare),
];
pub static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
```

这条规则不读取配置参数。工厂成功返回后，绑定层会拒绝多余参数。映射器只生成固定代码 `text.blank`，不把被拒绝的名称或领域错误文字放入违规项。应用启动时注册并绑定一次规则，把得到的 `BoundValidator` 放入资料服务，不要每次请求都重新绑定：

<!-- validator-guide:startup -->
```rust
// src/profile/startup.rs
use qubit_validator::{
    BoundValidator, InputType, RegistrationSource, ValidatorId,
    ValidatorRegistration, ValidatorRegistry,
};
use crate::profile::rules::DESCRIPTOR;

pub fn bind_display_name_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.non_blank"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    let configured_rule_id = String::from("text.non_blank");
    Ok(registry.bind(configured_rule_id.as_str(), InputType::Text, &[])?)
}
```

`String::from` 用来模拟从配置读取到的拥有型字符串；实际配置来源由应用提供。绑定接受运行时配置字符串，不要求它具有 `'static` 生命周期，也不要求先把它构造成 `ValidatorId`。`from_registrations` 会检查重复 ID 和无效描述符；`bind` 选择文本签名并准备规则。开放资料修改入口前应处理这些错误。这里只返回一条已绑定规则；规则较多时，应用可以持有注册表，在启动时逐条绑定配置选中的 ID。

收到请求后，资料服务借用输入名称执行规则，并把结果记在自己指定的字段路径：

<!-- validator-guide:service -->
```rust
// src/profile/service.rs
use qubit_validator::{
    BoundValidationContext, BoundValidator, ValidationPath,
    ValidationReport, ValidationValue,
};

fn check_display_name(
    rule: &BoundValidator,
    display_name: &str,
) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let outcome = rule.validate(
        ValidationValue::Text(display_name),
        &BoundValidationContext::new(&[]),
    )?;
    let mut report = ValidationReport::new();
    report.record_outcome(
        0,
        ValidationPath::root().with_field("display_name"),
        outcome,
    )?;
    Ok(report)
}
```

`0` 是本次请求验证计划中的检查序号。库不会检查资料结构体来推断字段。传入 `"Ada"` 时报告有效；传入 `"   "` 时，报告在 `display_name` 路径上包含一条 `text.blank` 违规。资料服务应先查看报告，再决定是否调用自己的存储接口。

### 这一步用到的类型

| 类型 | 在资料修改中的作用 |
| --- | --- |
| `Validator<str>` | 实现业务规则及其领域错误。 |
| `ValidatorSignature` / `ValidatorDescriptor` | 声明文本输入、依赖与准备函数。 |
| `ValidatorRegistry` | 根据稳定 ID 找到规则并绑定选中的签名。 |
| `BoundValidator` | 复用已准备的规则并检查输入形状。 |
| `ValidationOutcome` | 表示执行后有效、违规或跳过。 |
| `ValidationReport` | 按应用提供的路径和序号汇总结果。 |

## 保存前检查结果

`ProfileStore` 和 HTTP handler 由应用提供。先确认预定的整次验证均成功返回，再依据报告决定是否保存；执行错误携带的部分报告只用于诊断：

<!-- validator-guide:service-continuation -->
```rust
// 接续 src/profile/service.rs；check_display_name 已在上文定义。
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
    rule: &BoundValidator,
    user_id: &str,
    incoming_display_name: &str,
) -> Result<UpdateProfileResult, Box<dyn std::error::Error>> {
    let report = check_display_name(rule, incoming_display_name)?;
    if report.is_truncated() {
        return Err(std::io::Error::other("validation report is incomplete").into());
    }
    if !report.is_valid() {
        return Ok(UpdateProfileResult::Rejected(report));
    }
    store.save_display_name(user_id, incoming_display_name)?;
    Ok(UpdateProfileResult::Updated)
}
```

存储接口与 `UpdateProfileResult` 都是应用代码。`"Ada"` 会进入 `save_display_name`，该调用成功后返回 `Updated`；全空白名称返回 `Rejected(report)`，不会访问存储。存储失败仍是应用错误。handler 应先检查 `is_truncated()`，再遍历 `report.violations()`，将违规代码映射为受控文案。无效报告也可能没有已保留的违规项，不要直接索引第一项。模型校验集成若在执行错误中附带部分报告，应保留用于诊断，不能据此放行写入。请求中的名称不能进入文案或日志。

| 发生了什么 | 检查什么 | 应用下一步 |
| --- | --- | --- |
| `"Ada"` 通过 | 预定检查全部成功完成后，`report.is_valid()` 为 true | 调用应用自己的存储层。 |
| `"   "` 违反规则 | `ValidationOutcome::Invalid`，报告含 `text.blank` | 拒绝修改，把代码映射为字段提示。 |
| ID、签名或参数配置有误 | 启动阶段的 `ValidatorRegistryError` 或 `BindError` | 修正配置，再开放请求入口。 |
| 输入形状或规则执行出错 | `validate` 返回 `Err(ExecutionError)` | 作为服务故障处理，走可信诊断流程；不要当作字段错误。 |
| 有限额的报告装不下全部结果 | `!recorded.complete()` 或 `report.is_truncated()` | 视为结果不完整，不根据缺失的违规项放行。 |

`record_outcome` 返回 `RecordedOutcome`；`complete()` 表示本次结果完整写入报告，**不表示验证通过**。上例用 `ValidationReport::new()`，没有条数上限。若设置限额，要同时检查完整性、截断状态和有效性。`ValidationOutcome::Invalid` 表示规则已执行且值不符合要求，不属于 `ExecutionError`。所有情况下都应先确认预定的整次验证成功返回；执行错误的部分报告只供诊断，不能用于放行。

至此，基础路径完成：启动时绑定，请求中验证，通过后才保存。后续仍围绕资料修改，按需要扩展这条路径。

## 比较资料中的两个字段

如果资料表单同时包含 `email` 和 `email_confirmation`，确认邮箱规则需要读取同一请求里的邮箱。把依赖声明在签名和适配器中，启动时绑定；下面这个独立模块包含规则定义、启动时绑定和请求时调用：

<!-- validator-guide:confirmation -->
```rust
// src/profile/confirmation.rs
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, BoundValidator, DependencySpec, ExecutionError,
    InputType, NamedValidationDependency,
    PreparedOutcome, PreparedValidator, RegistrationSource, ValidationOutcome,
    ValidationValue, ValidatorDescriptor, ValidatorId, ValidatorRegistration,
    ValidatorRegistry, ValidatorSignature, ViolationCode, ViolationDraft,
    prepare_text_with_context,
};

static EMAIL_DEPENDENCIES: &[DependencySpec] = &[
    DependencySpec::new("email", InputType::Text, false),
];

fn prepare_confirmation(_reader: &mut ArgumentReader<'_>)
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    Ok(prepare_text_with_context(EMAIL_DEPENDENCIES, |confirmation, context| {
        let email = context.text(0)?;
        Ok(if confirmation == email {
            PreparedOutcome::Valid
        } else {
            PreparedOutcome::Invalid(vec![ViolationDraft::new(
                ViolationCode::new("email.confirmation_mismatch"),
            )])
        })
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, EMAIL_DEPENDENCIES, prepare_confirmation),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

pub fn bind_confirmation_rule() -> Result<BoundValidator, Box<dyn std::error::Error>> {
    let registration = ValidatorRegistration::new(
        ValidatorId::new("email.confirmation"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    Ok(registry.bind("email.confirmation", InputType::Text, &[])?)
}

pub fn check_confirmation(
    rule: &BoundValidator,
    email: &str,
    confirmation: &str,
) -> Result<ValidationOutcome, ExecutionError> {
    let dependencies = [NamedValidationDependency::new(
        "email", ValidationValue::Text(email),
    )];
    rule.validate_named(ValidationValue::Text(confirmation), &dependencies)
}
```

启动时调用 `bind_confirmation_rule()` 并把结果交给资料服务。收到请求后，调用 `check_confirmation(&rule, email, confirmation)`，再把结果记在 `email_confirmation` 路径。规则较多时，也可以把多个描述符放进同一个注册表。适配器通过 `context.text(0)?` 读取已声明的槽位；无法读取时返回执行错误。

两个邮箱相同返回 `Valid`，不同则产生 `email.confirmation_mismatch`；缺少必填依赖或类型错误返回 `ExecutionError`，应走服务错误分支。`validate_named` 按签名名称匹配槽位：零个或一个依赖时不会分配重排容器；两个到八个依赖使用固定容量的临时缓冲；九个及以上依赖才会按需分配缓冲。克隆非根路径 `ValidationPath` 时仍可能分配。若上游已经核验顺序，也可使用 `validate` 和 `BoundValidationContext::new(&values)`。可选依赖仍占一个槽位，缺失时显式传 `ValidationValue::Missing`；错误类型不能当作缺失。

## 在启动时配置规则

如果资料策略限制显示名称长度，可以在绑定时读取 `maximum`，之后每次请求复用该上限。下面的独立模块同时包含规则准备和启动时绑定；它按 Unicode 标量值计数，若应用需要按字节或用户感知字符计数，应另外定义规则。

<!-- validator-guide:max-length -->
```rust
// src/profile/max_length.rs
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, BoundValidator, InputType, NamedValidationArgument,
    PreparedOutcome, PreparedValidator, RegistrationSource, ValidationArgument,
    ValidatorDescriptor, ValidatorId, ValidatorRegistration, ValidatorRegistry,
    ValidatorSignature, ViolationCode, ViolationDraft, prepare_text_with_context,
};

fn prepare_max_length(reader: &mut ArgumentReader<'_>)
    -> Result<Arc<dyn PreparedValidator>, BindError>
{
    let maximum = reader.required_u32("maximum")?;
    Ok(prepare_text_with_context(&[], move |name, _| {
        Ok(if name.chars().count() <= maximum as usize {
            PreparedOutcome::Valid
        } else {
            PreparedOutcome::Invalid(vec![ViolationDraft::new(
                ViolationCode::new("text.too_long"),
            )])
        })
    }))
}

static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_max_length),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

pub fn bind_max_length_rule(maximum: u32)
    -> Result<BoundValidator, Box<dyn std::error::Error>>
{
    let registration = ValidatorRegistration::new(
        ValidatorId::new("text.max_length"),
        &DESCRIPTOR,
        RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    let args = [NamedValidationArgument::new(
        "maximum", ValidationArgument::Unsigned(u128::from(maximum)),
    )];
    Ok(registry.bind("text.max_length", InputType::Text, &args)?)
}
```

启动时调用 `bind_max_length_rule(40)`。资料服务用已绑定的长度规则验证 `display_name`，并以接下来的检查序号把结果写到同一路径。长度不超过 40 个标量值时通过，否则产生 `text.too_long`。缺少参数、参数类型或范围不对、存在未读取参数，都会在绑定阶段报错。`ArgumentReader` 首次按类型读取时就会消耗已提供的参数，即使转换失败也一样；修正配置后重新绑定，不要反复读取同一个 reader。

## 汇总多项检查并处理容量上限

资料服务自己决定规则执行顺序。调用 `record_outcome` 时序号不能递减；同一个序号可以记录多条结果。传入更小的序号会返回 `ValidationOutcomeError::OutOfOrderOccurrence`，报告保持不变。例如 `email` 已为空白时，应用可以跳过确认邮箱检查，并让跳过记录引用原始错误：

```rust
// non_blank_rule 是启动时绑定的 text.non_blank 规则。
use qubit_validator::{
    BoundValidationContext, ValidationOutcome, ValidationPath,
    ValidationReport, ValidationValue,
};
let mut report = ValidationReport::new();
let email_outcome = non_blank_rule.validate(
    ValidationValue::Text("   "),
    &BoundValidationContext::new(&[]),
)?;
let original = report.record_outcome(
    0,
    ValidationPath::root().with_field("email"),
    email_outcome,
)?;
let skipped = report.record_outcome(
    1,
    ValidationPath::root().with_field("email_confirmation"),
    ValidationOutcome::failed_prerequisite(original.failure_ids().to_vec())?,
)?;
assert!(original.complete() && skipped.complete());
assert_eq!(report.failure_count(), 1);
assert_eq!(report.skipped().len(), 1);
```

只有前一步确实保留了违规项时，才能调用 `failed_prerequisite`。`FailureId` 只属于当前报告，指向已记录的原始违规；`report.failures()` 只遍历原始失败，不会因跳过而重复计数。是否跳过由应用决定，库不会替应用调度规则。可选目标缺失时，用 `ValidationOutcome::missing_optional()` 记录另一种跳过原因。

对检查项较多的请求，可用 `ValidationReport::with_limits(ValidationLimits { max_violations: Some(20), max_skipped: Some(20) })` 分别限制保留的违规和跳过记录。跳过项引用已有失败，不再占用违规项额度。检查每次返回的 `RecordedOutcome::complete()` 和 `report.is_truncated()`；应用若提前停止检查，还应调用 `report.mark_truncated()`。截断报告不能证明后续字段都通过。这些上限不限制输入长度、验证计算量、依赖路径、先决条件列表或记录前已分配的结果；应用需要另设这些预算。

若执行发生错误且集成返回已收集的部分报告，只能用它诊断已有证据。只有确认预定的整次验证成功完成后，才能依据 `is_valid()` 放行操作。

## 选择注册方式和适配器

`ValidatorRegistry::from_registrations` 让应用明确控制有哪些规则，也可在一个进程中建立不同规则集。如果确实需要由已链接的其他 crate 向进程级注册表提交规则，再启用可选功能：

```toml
[dependencies]
qubit-validator = { version = "0.1", features = ["inventory"] }
```

在规则定义处使用 `register_validator!`，启动时调用 `ValidatorRegistry::try_global()`。重复 ID 或无效描述符会返回错误；`global()` 遇到这些错误会 panic。发现规则的方式变了，绑定和执行契约没有改变。如果同一个 `ValidatorRegistration` 常量还要用于局部注册表，可传入 `registration = SHARED_REGISTRATION`；这种写法会复用该值已有的 ID、描述符和来源信息。`id`/`descriptor` 写法则记录宏调用位置的来源。两种写法都要求启用 `inventory` 功能且 ID 静态有效；`id` 写法中的字面量会在编译期检查。共享注册示例见 [`register_validator!` API 文档](https://docs.rs/qubit-validator/latest/qubit_validator/macro.register_validator.html)。[仓库示例](../examples/local_registry.rs)也展示了启用该功能时的路径。

简单的 `prepare_text_validator`、`prepare_typed_validator` 将带类型规则的领域错误映射成一条违规草稿；读取依赖时使用对应的上下文适配器。领域错误会在映射期间被消费，因此无需满足 `Send + Sync`；被存储的验证器、领域调用闭包、映射器闭包和预备适配器仍保留各自的线程安全约束。若规则需要一次返回多条违规，或需要把外部故障与输入不合规则区分，可用 `prepare_text_with_context` / `prepare_typed_with_context` 返回 `PreparedOutcome` 或 `ExecutionError`；[`prepare_text_domain_rule`](https://docs.rs/qubit-validator/latest/qubit_validator/fn.prepare_text_domain_rule.html) / [`prepare_typed_domain_rule`](https://docs.rs/qubit-validator/latest/qubit_validator/fn.prepare_typed_domain_rule.html) 可把带上下文的领域错误映射成安全草稿。规则验证精确 Rust 类型时选择类型化工厂，输入为字符串时选择文本工厂。表示违规的草稿列表不能为空。已有无依赖、精确接受 `T` 的预备规则，可用 `BoundValidator::try_from_prepared::<T>` 跳过注册表直接绑定。

下面的目录查询是应用提供的模拟实现，不代表真实仓储。名字长度上限属于规则配置，可作为安全报告元数据。领域拒绝会在报告中生成两条违规；查询故障直接返回 `ExecutionError`，不会写入报告。只有所有预定检查都成功完成后，才能用 `is_valid()` 决定是否保存。文档示例测试会编译并运行这段程序。

<!-- validator-guide:domain-rule -->
```rust
use std::sync::Arc;
use qubit_validator::{
    ArgumentReader, BindError, DependencySpec, ExecutionError, ExecutionErrorKind,
    InputType, NamedValidationDependency, PreparedValidator, RegistrationSource,
    ValidationOutcome, ValidationPath, ValidationReport, ValidationValue,
    ValidatorDescriptor, ValidatorId, ValidatorRegistration, ValidatorRegistry,
    ValidatorSignature, ViolationCode, ViolationDraft, ViolationParam,
    prepare_text_domain_rule,
};

// Application-provided stand-in for a directory lookup.
fn directory_available(email: &str) -> Result<bool, ExecutionError> {
    if email == "offline@example.test" {
        Err(ExecutionError::new(ExecutionErrorKind::ExternalFailure))
    } else {
        Ok(true)
    }
}
fn prepare(_: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_text_domain_rule(
        DEPENDENCIES, 40_usize,
        |maximum, name, context| {
            let email = context.text(0)?;
            let available = directory_available(email)?;
            let domain_result = if available && !name.trim().is_empty()
                && name.chars().count() <= *maximum
            {
                Ok(())
            } else {
                Err(std::io::Error::other("profile policy rejected"))
            };
            Ok((domain_result, *maximum))
        },
        |_, maximum| vec![
            ViolationDraft::new(ViolationCode::new("profile.name_policy"))
                .with_param("maximum", ViolationParam::Unsigned(*maximum as u128)),
            ViolationDraft::new(ViolationCode::new("profile.update_rejected")),
        ],
    ))
}
static DEPENDENCIES: &[DependencySpec] = &[DependencySpec::new("email", InputType::Text, false)];
static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, DEPENDENCIES, prepare),
];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = ValidatorId::new("profile.name_policy");
    let registration = ValidatorRegistration::new(
        id, &DESCRIPTOR,
        RegistrationSource::new("profile-app", "profile", file!(), line!()),
    );
    let registry = ValidatorRegistry::from_registrations([registration])?;
    let rule = registry.bind("profile.name_policy", InputType::Text, &[])?;
    let dependencies = [NamedValidationDependency::new(
        "email", ValidationValue::Text("ada@example.test"),
    )];
    assert_eq!(rule.validate_named(ValidationValue::Text("Ada"), &dependencies)?, ValidationOutcome::Valid);
    let rejected = rule.validate_named(ValidationValue::Text("   "), &dependencies)?;
    let mut report = ValidationReport::new();
    let receipt = report.record_outcome(
        0, ValidationPath::root().with_field("display_name"), rejected,
    )?;
    assert!(receipt.complete());
    assert!(!report.is_truncated());
    assert!(!report.is_valid());
    assert_eq!(report.failure_count(), 2);
    assert_eq!(report.violations()[0].rule_id(), id);
    assert_eq!(report.violations()[0].path().render(), "display_name");
    assert_eq!(report.violations()[0].code().as_str(), "profile.name_policy");
    assert_eq!(report.violations()[1].code().as_str(), "profile.update_rejected");
    let should_save = report.is_valid();
    assert!(!should_save);
    let offline = [NamedValidationDependency::new(
        "email", ValidationValue::Text("offline@example.test"),
    )];
    assert_eq!(rule.validate_named(ValidationValue::Text("Ada"), &offline)
        .unwrap_err().kind(), ExecutionErrorKind::ExternalFailure);
    assert_eq!(report.failure_count(), 2);
    assert_eq!(rule.validate_named(ValidationValue::Text("Ada"), &[])
        .unwrap_err().kind(), ExecutionErrorKind::MissingDependencyBinding);
    assert_eq!(report.failure_count(), 2);
    Ok(())
}
```

## 选择稳定的规则 ID 和违规代码

`ValidatorId` 用来标识规则，`ViolationCode` 用来标识报告中的违规类型。两者都必须是非空字符串，由一个或多个点分隔的片段组成。每段以 ASCII 字母开头，后续字符只能是 ASCII 字母、数字或下划线。空片段、空白字符、非 ASCII 字母和连字符都会被拒绝；构造时不会去掉首尾空白，也不会转换大小写。例如，`app.valid_rule` 和 `text.length.minimum` 有效；`bad..id`、`_custom.invalid` 和 `text.min-length` 无效。

`ValidatorId::try_new` 和 `ViolationCode::try_new` 都要求 `&'static str`；它们处理的是语法校验失败，不用于运行时配置字符串。对于程序声明且已有静态存储、但语法可能无效的 ID 或违规代码，也可以用它们获取校验错误。运行时 ID 用 `ValidatorRegistry::get` 或 `bind` 选择规则；绑定未知 ID 时，`bind` 返回 `MissingRule`，调用方保留请求文本。绑定示例见[在启动时配置规则](#在启动时配置规则)。违规代码仍由程序静态声明，不要把配置字符串暴露为违规代码。`new` 遇到无效值会 panic。传给 `register_validator!` 的字面量 ID 会在编译期检查。完整错误变体和函数签名见 [`ValidatorId` API 文档](https://docs.rs/qubit-validator/latest/qubit_validator/struct.ValidatorId.html) 与 [`ViolationCode` API 文档](https://docs.rs/qubit-validator/latest/qubit_validator/struct.ViolationCode.html)。

## 排查故障并守住边界

| 现象 | 检查位置与处理 |
| --- | --- |
| 启动时报 `MissingRule` | 核对 ID 是否在选用的注册表中；使用 `inventory` 时确认已启用功能且规则所在 crate 已链接。 |
| `UnsupportedInput` 或 `InputTypeMismatch` | 对照签名的 `InputType` 和调用时的 `ValidationValue`；文本不会自动转换成类型化值。 |
| `UnknownParameter` 或 `ParameterAlreadyConsumed` | 每个参数只读一次，最后调用 `finish()`；修正配置后重新绑定。 |
| 依赖检查失败 | 核对槽位名称、顺序、类型和必填状态；只有可选值缺失才使用 `Missing`。 |
| `AdapterContractViolation` | 检查自定义预备规则声明的输入与依赖，以及违规结果是否含草稿。 |
| `!complete()` 或 `is_truncated()` | 检查报告上限和提前退出的分支；将结果视为不完整。 |
| 日志中看不到字段名 | `ValidationPath` 的默认格式隐藏字段与 map 位置；只有允许披露的展示层才调用 `render()`。 |

`ExecutionError` 保留的底层原因只能由可信诊断代码通过 `trusted_source()` 读取；普通 `Display`、`Debug` 和 `Error::source()` 不会暴露它。`ViolationParam` 只限制诊断值的表示类型，不能证明值来自哪里：配置里的数字可能适合输出，从被拒绝输入推导出的同类型数字则可能泄露信息。不要把被拒绝的值或敏感数据写入代码、参数、错误或日志。若客户端依赖规则 ID 和违规代码，变更时要同步调用方。

规则同步执行，每次调用只借用输入值；预备规则要求 `Send + Sync`。本库不创建线程，也不依赖异步运行时。对象遍历、规则调度、请求处理、数据存储与文案本地化都由应用负责。

## 延伸阅读

- [中文 README](../README.zh_CN.md) · [设计与不变量](design.zh_CN.md) · [API 文档](https://docs.rs/qubit-validator)
- [可运行的局部注册表示例](../examples/local_registry.rs) · [English user guide](user_guide.md)
