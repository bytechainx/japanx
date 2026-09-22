# japanx 公开 API

> 本文件列出 `japanx` 的公开面：类型 / 方法 + 一行语义。
> 公开面**不**声称任何能力等级、SLA、新鲜度保证或生产就绪。

## 错误模型

| 项 | 语义 |
| --- | --- |
| `JapanCbErrorKind` | 错误分类：`Invalid` / `Missing` / `AuthorizationDenied` / `RoutedElsewhere` / `WriteAuthorityDenied` / `SemanticallyRejected` / `NotApplicable` / `Invariant` |
| `JapanCbError` | 类型化错误，`#[non_exhaustive]`，消息为简体中文且不回显原文 |
| `JapanCbError::kind()` | 分类，供调用方按「如何反应」分流 |
| `JapanCbError::is_retryable()` | 本层无网络，除 `Invariant` 外一律 `false` |
| `JapanCbResult<T>` | `Result<T, JapanCbError>` |

## metric_code

| 项 | 语义 |
| --- | --- |
| `DECLARED_METRIC_CODES` | 清单 §1.1 的 12 个 `JP.*` metric_code |
| `JP_BS_TOTAL_ASSETS` … `JP_TK_LARGE_MFG_DI` | 12 个具名常量 |
| `validate_metric_code` | 校验规范形 `JP.<类别>.<指标>[.<子维度>]` |
| `is_declared_metric_code` | 是否属于清单声明的 12 个 |
| `JapanCbMetricCode` | 校验后的 metric_code |
| `JapanCbMetricCode::as_str` / `category` / `is_declared` | 原文 / 类别 / 是否声明 |
| `MetricCategory` | 8 个类别码 `BS`/`IR`/`MB`/`MS`/`CA`/`MO`/`FX`/`TK` |
| `MetricCategory::ALL` / `code` / `from_code` | 全集与类别码互转 |

## 期间与频率

| 项 | 语义 |
| --- | --- |
| `Period` | `Day(Date)` / `TenDay{year,month,segment}` / `Month` / `Quarter` / `Year` / `Event` |
| `Period::day` / `ten_day` / `month` / `quarter` / `year` / `event` | 各形态构造（含范围校验） |
| `Period::parse` / `key` / `frequency` | 严格解析 / 规范键 / 最贴合频率 |
| `Frequency` | `Daily` / `TenDay` / `Weekly` / `Monthly` / `Quarterly` / `Annual` / `Event` / `Irregular` |
| `Frequency::ALL` / `SOURCE_DECLARED` / `name` / `from_name` / `from_source_token` | 全集 / 清单声明 5 值 / 名称互转 / 只认清单声明值 |
| `Date` | 自有日期（严格 `YYYY-MM-DD`，校验闰年） |
| `Date::new` / `parse` / `to_iso` / `year` / `month` / `day` / `is_leap_year` / `days_in_month` | 日期构造与解析 |

## 数据点

| 项 | 语义 |
| --- | --- |
| `Unit` | 源侧单位原文（`100 million yen` 原样保留） |
| `Unit::try_new` / `text` | 构造校验 / 原文 |
| `SOURCE_UNIT_HUNDRED_MILLION_YEN` | 清单声明的源侧单位 |
| `Revision` | 源侧修订标识原文 |
| `Revision::try_new` / `text` | 构造校验 / 原文 |
| `JapanCbMissingReason` | 缺失原因：`SourceStatus(原文)` / `Unspecified` |
| `DataPointValue` | `Value(f64)` / `Missing(JapanCbMissingReason)` |
| `DataPointValue::value` / `is_missing` / `missing_reason` | 取值 / 是否缺失 / 缺失原因 |
| `JapanCbDataPoint` | 一条源事实数据点 |
| `JapanCbDataPoint::new` / `data_point_id` | 构造并校验 / 身份串 |
| `validate_data_point` | 校验数据点完整性与粒度自洽 |

## 守卫（fail-closed）

| 项 | 语义 |
| --- | --- |
| `claim_curve_authority` | 恒 `RoutedElsewhere`：MOF / BOJ 曲线归 `yieldx` |
| `EXCLUDED_FX_SYMBOLS` / `FRED_EXCLUSIVE_FX_SERIES` | `JPY=X` / `USDJPY=X` 与 `DEXJPUS` |
| `reject_excluded_fx_symbol` | 恒 `WriteAuthorityDenied`：本库不得主张该主源 |
| `reject_silent_fx_substitution` | 近义非同 ID 不得互换（`SemanticallyRejected`） |
| `claim_forwarded_series_sovereignty` | 恒 `WriteAuthorityDenied`：JGB10Y / CPI 主权待裁 |
| `ensure_no_converted_or_derived_field` | 恒 `NotApplicable`：换算 / 派生 / 存储字段不属本层 |

## 授权判定

| 项 | 语义 |
| --- | --- |
| `JapanCbAuthorization` | `Authorized { scope }` / `Denied { reason }` |
| `JapanCbAuthorization::is_authorized` / `denial_reason` | 是否放行 / 拒绝理由 |
| `JapanCbAuthorizationEvidence` | 证据的离线登记形态 |
| `JapanCbAuthorizationEvidence::new` | 构造一条证据登记（不做放行判断） |
| `decide_authorization` | fail-closed 判定 |
| `registered_evidence` | 本源的证据登记：恒 `None` |
| `current_authorization` | 本源当前判定：恒 `Denied` |
| `ensure_authorized` | `Denied` → `AuthorizationDenied` 错误 |

## publication 语义

| 项 | 语义 |
| --- | --- |
| `TimePrecision` | `Date` / `Instant` |
| `AvailabilityEvidence` | `Official` / `Calendar` / `Inferred` |
| `PitEligibility` | `Formal` / `NotEligible` |
| `publication_semantics` | 恒 `(Date, Inferred, NotEligible)` |
| `publication_for_period` | 三元组随期间一并返回（期间不参与判定） |

## 离线解析

| 项 | 语义 |
| --- | --- |
| `parse_japanx_data_points` | JSON 文本 → 数据点集合；未知字段原子失败、重复身份拒绝 |
| `parse_japanx_source_definition` | TOML 文本 → 源定义；未知字段（含限流数字）原子失败 |
| `JapanCbSourceDefinition` | 源定义（`kind` / `encoding` / `unit` / `scale` / `metric_codes`） |
| `SOURCE_KIND_CSV_DOWNLOAD` | 清单声明的源形态 `csv_download` |
| `SOURCE_ENCODING_SHIFT_JIS` | 清单声明的源侧编码 `shift_jis` |
| `PARSER_INPUT_ENCODING` | 本层要求的输入编码 `utf-8`（源文本须先转码） |
