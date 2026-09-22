# japanx

`japanx` 是日本央行（BOJ）**源事实的类型层**：把清单里已钉死的 `JP.*` metric_code、
期间、单位落成可机械校验的 Rust 类型，并提供离线解析与 fail-closed 授权判定。

- 清单 §1.1 的 **12 个 metric_code** 常量与声明集（`JP.BS.TOTAL_ASSETS` …
  `JP.TK.LARGE_MFG_DI`）
- 规范形校验 `JP.<类别>.<指标>[.<子维度>]`；8 个类别码
  （`BS` / `IR` / `MB` / `MS` / `CA` / `MO` / `FX` / `TK`）
- 期间与频率覆盖清单的 `Daily|TenDay|Monthly|Quarterly|Annual`
- 源侧单位 `100 million yen` **原样保留**，不做换算
- 离线解析：JSON 数据点（未知字段原子失败、重复身份拒绝）+ TOML 源定义
- 三条 fail-closed 守卫：曲线归 `yieldx`、`JPY=X` / `USDJPY=X` 归 `fredx` 独占、
  `unknown` 授权一律 `Denied`
- **不写具体限流数字**：源定义模型不含限流字段，`rate_limit_rps = <数字>` 会原子失败
- 零 HTTP 依赖、零端点字面量、零凭据读取、零跨仓依赖

## 安装

本 crate **不发布到 crates.io**，通过 git 依赖引入：

```toml
[dependencies]
japanx = { git = "https://github.com/bytechainx/japanx" }
```

## 用法示例

构造一条数据点：

```rust,no_run
use japanx::{
    validate_metric_code, DataPointValue, Frequency, JapanCbDataPoint, Period, Unit,
    JP_BS_TOTAL_ASSETS, SOURCE_UNIT_HUNDRED_MILLION_YEN,
};

fn main() -> Result<(), japanx::JapanCbError> {
    let point = JapanCbDataPoint::new(
        validate_metric_code(JP_BS_TOTAL_ASSETS)?,
        Period::parse("2026-08")?,
        Frequency::Monthly,
        DataPointValue::Value(1.0),
        Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN)?,
        None,
    )?;
    assert_eq!(point.data_point_id(), "JP.BS.TOTAL_ASSETS:2026-08:none");
    Ok(())
}
```

离线解析（参数只有文本，**不接受** URL、客户端或认证信息）：

```rust,no_run
use japanx::parse_japanx_data_points;

fn main() -> Result<(), japanx::JapanCbError> {
    let document = r#"{ "_synthetic": true, "_note": "合成样本", "data_points": [] }"#;
    assert!(parse_japanx_data_points(document)?.is_empty());
    Ok(())
}
```

## 主要内容

| 类型 / 函数 | 作用 |
| --- | --- |
| `DECLARED_METRIC_CODES` / `JP_*` 常量 | 清单 §1.1 的 12 个 metric_code |
| `validate_metric_code` | 规范形 `JP.<类别>.<指标>[.<子维度>]` 校验 |
| `JapanCbMetricCode` / `MetricCategory` | 校验后的码与 8 个类别 |
| `Period` / `Frequency` | 期间（含旬 `TenDay`）与频率 |
| `Unit` / `Revision` | 源侧单位与修订标识原文 |
| `DataPointValue` / `JapanCbMissingReason` | 有值 / 具名缺失（绝不转 0） |
| `JapanCbDataPoint` | 主观测类型，含 `data_point_id` |
| `validate_data_point` | 数据点完整性与粒度自洽校验 |
| `parse_japanx_data_points` | 离线 JSON → 数据点集合 |
| `parse_japanx_source_definition` / `JapanCbSourceDefinition` | TOML 源定义（`kind=csv_download`） |
| `claim_curve_authority` | 恒 `RoutedElsewhere`：曲线归 `yieldx` |
| `reject_excluded_fx_symbol` / `reject_silent_fx_substitution` | `JPY=X` / `USDJPY=X` 归 `fredx` 独占 |
| `claim_forwarded_series_sovereignty` | 恒 `WriteAuthorityDenied`：JGB10Y / CPI 主权待裁 |
| `ensure_no_converted_or_derived_field` | 恒 `NotApplicable`：`value_usd` / `fingerprint` / `raw_object_key` 不属本层 |
| `JapanCbAuthorization` / `decide_authorization` / `current_authorization` | fail-closed 授权判定 |
| `publication_semantics` | 恒为 `Date` + `Inferred` + `NotEligible` |

## 非目标

- **不实现联网采集**：不含任何端点字面量、Header、凭据字段；统计码 / URL / 许可全部 UNKNOWN
- **不写限流数字**：授权前不写具体限流值，任何 `rate_limit_rps = <数字>` 都会被原子拒绝
- **不实装 Shift-JIS 编解码**：本层只接受已转码的 UTF-8 文本，不引入编码库
- **不主张 `JPY=X` / `USDJPY=X`**：两者归 `fredx` 独占（`DEXJPUS`）
- **不把 MOF / BOJ 曲线晋级**：曲线归 `yieldx` #13–14
- **不做单位换算与派生指标**：`value_usd` / `fingerprint` / `raw_object_key` 均不属本层
- **不编造统计码全集**：清单只给了 12 个 metric_code，没有官方 stat_code 字典
- **不读凭据 / 环境变量**：本层只做离线解析

## 诚实边界

`production_decision = NO-GO`。清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
本源**无 Owner 签核文件**（`authorization = unknown`），因此 `current_authorization` 恒返回
`Denied`——本库不假装有签核。

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

全部用例离线运行，不访问任何外部服务。

## 许可

MIT OR Apache-2.0
