#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! japanx —— 日本央行（BOJ）源事实的类型层：12 个 `JP.*` metric_code、期间与单位、
//! 离线解析与 fail-closed 授权判定。
//!
//! ## 能力
//!
//! | 能力 | 状态 |
//! | --- | --- |
//! | 12 个 `JP.*` metric_code 常量与声明集 | 已实现 |
//! | 规范形校验 `JP.<类别>.<指标>[.<子维度>]` | 已实现 |
//! | 8 个类别码（`BS`/`IR`/`MB`/`MS`/`CA`/`MO`/`FX`/`TK`） | 已实现 |
//! | 期间（含清单的 `TenDay` 旬）与频率 | 已实现 |
//! | 源侧单位 `100 million yen` 原样保留 | 已实现（不做换算） |
//! | 离线 JSON 数据点解析 + TOML 源定义解析 | 已实现 |
//! | 授权判定（fail-closed） | 已实现；本源 `authorization = unknown`，判定恒 `Denied` |
//! | 曲线晋级 / `JPY=X` 主源主张 / live 采集 | **未实现且禁止** |
//!
//! ## 责任边界
//!
//! 本库做：把清单里的源事实落成可机械校验的类型；对曲线路由、禁抢主源与授权
//! 三类越界行为 fail-closed 拒绝。
//! 本库不做：联网采集、认证、缓存、再分发、存储、单位换算、派生指标、编码转码。
//!
//! ## 非目标
//!
//! - 不实现 HTTP / 采集器：统计码、URL、许可全部 UNKNOWN，**规划端点 ≠ 访问合同**
//! - 不写具体限流数字：源定义模型不含限流字段，任何 `rate_limit_rps = <数字>` 都会原子失败
//! - 不实装 Shift-JIS 编解码：本层只接受已转码的 UTF-8 文本（格式契约见 `docs/标准.md`）
//! - 不主张 `JPY=X` / `USDJPY=X`：两者归 `fredx` 独占（`DEXJPUS`）
//! - 不把 MOF / BOJ 曲线晋级：归 `yieldx` #13–14
//! - 不做派生指标（净流动性、利差、Credit Impulse、z-score）
//!
//! ## 诚实边界
//!
//! `production_decision = NO-GO`；清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
//! 本源无 Owner 签核文件，`authorization = unknown`，本库**不假装**有签核。
//! 清单只声明了 12 个 metric_code，**没有**官方 stat_code 字典——本库不编造。
//!
//! # Examples
//!
//! ```
//! # fn main() -> Result<(), japanx::JapanCbError> {
//! use japanx::{
//!     validate_metric_code, DataPointValue, Frequency, JapanCbDataPoint, Period, Unit,
//!     JP_BS_TOTAL_ASSETS, SOURCE_UNIT_HUNDRED_MILLION_YEN,
//! };
//!
//! let metric_code = validate_metric_code(JP_BS_TOTAL_ASSETS)?;
//! assert!(metric_code.is_declared());
//!
//! let point = JapanCbDataPoint::new(
//!     metric_code,
//!     Period::parse("2026-08")?,
//!     Frequency::Monthly,
//!     DataPointValue::Value(1.0),
//!     Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN)?,
//!     None,
//! )?;
//! assert_eq!(point.data_point_id(), "JP.BS.TOTAL_ASSETS:2026-08:none");
//! // 授权判定 fail-closed：本源无签核文件，故恒为 Denied。
//! assert!(!japanx::current_authorization(japanx::Date::new(2026, 9, 22)?).is_authorized());
//! # Ok(())
//! # }
//! ```

pub mod authz;
pub mod error;
pub mod parse;
pub mod pit;
pub mod value;

pub use authz::{
    current_authorization, decide_authorization, ensure_authorized, registered_evidence,
    JapanCbAuthorization, JapanCbAuthorizationEvidence,
};
pub use error::{JapanCbError, JapanCbErrorKind, JapanCbResult};
pub use parse::{
    parse_japanx_data_points, parse_japanx_source_definition, JapanCbSourceDefinition,
    PARSER_INPUT_ENCODING, SOURCE_ENCODING_SHIFT_JIS, SOURCE_KIND_CSV_DOWNLOAD,
};
pub use pit::{
    publication_for_period, publication_semantics, AvailabilityEvidence, PitEligibility,
    TimePrecision,
};
pub use value::{
    claim_curve_authority, claim_forwarded_series_sovereignty,
    ensure_no_converted_or_derived_field, is_declared_metric_code, reject_excluded_fx_symbol,
    reject_silent_fx_substitution, validate_data_point, validate_metric_code, DataPointValue, Date,
    Frequency, JapanCbDataPoint, JapanCbMetricCode, JapanCbMissingReason, MetricCategory, Period,
    Revision, Unit, DECLARED_METRIC_CODES, EXCLUDED_FX_SYMBOLS, FRED_EXCLUSIVE_FX_SERIES,
    JP_BS_ETF_HOLDINGS, JP_BS_JGB_HOLDINGS, JP_BS_TOTAL_ASSETS, JP_CA_EXCESS_RESERVES, JP_CA_TOTAL,
    JP_FX_INTERVENTION, JP_IR_POLICY_RATE, JP_IR_TONA, JP_MB_TOTAL, JP_MO_JGB_PURCHASE, JP_MS_M2,
    JP_TK_LARGE_MFG_DI, SOURCE_UNIT_HUNDRED_MILLION_YEN,
};
