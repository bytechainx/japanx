# Changelog — japanx

本文件记录 `japanx` 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)
与 [Semantic Versioning](https://semver.org/)。

## [0.1.0] - 2026-09-22

### Added

- 清单 §1.1 的 **12 个 `JP.*` metric_code**：`DECLARED_METRIC_CODES` 与 12 个具名常量
  （`JP_BS_TOTAL_ASSETS` … `JP_TK_LARGE_MFG_DI`）。
- 规范形校验 `validate_metric_code`（`JP.<类别>.<指标>[.<子维度>]`）与
  `JapanCbMetricCode` / `is_declared_metric_code`。
- 类别枚举 `MetricCategory`：`BS` / `IR` / `MB` / `MS` / `CA` / `MO` / `FX` / `TK`。
- 期间 `Period`（契约五变体 + 清单的旬 `TenDay`）与频率 `Frequency`
  （契约七取值 + `TenDay`）；`Frequency::from_source_token` 只接受清单声明的 5 个取值。
- 值对象：`Unit`（源侧单位 `100 million yen` 原样保留）、`Revision`（修订标识原文）、
  `DataPointValue` / `JapanCbMissingReason`（具名缺失，绝不静默转 0）、
  `JapanCbDataPoint`（含 `data_point_id`）、`validate_data_point`。
- 离线解析：`parse_japanx_data_points`（JSON，未知字段原子失败、重复身份拒绝、
  要求显式合成样本标注）与 `parse_japanx_source_definition`（TOML，`kind=csv_download`）。
- fail-closed 守卫：`claim_curve_authority`（曲线归 `yieldx`）、
  `reject_excluded_fx_symbol` / `reject_silent_fx_substitution`
  （`JPY=X` / `USDJPY=X` 归 `fredx` 独占）、`claim_forwarded_series_sovereignty`
  （JGB10Y / CPI 主权待裁）、`ensure_no_converted_or_derived_field`
  （`value_usd` / `fingerprint` / `raw_object_key` 不属本层）。
- 授权判定：`JapanCbAuthorization` / `decide_authorization` / `current_authorization` /
  `ensure_authorized`；本源 `authorization = unknown`，判定恒 `Denied`。
- publication 语义 `publication_semantics`：恒为 `Date` + `Inferred` + `NotEligible`。
- 三类测试（`tests/tdd_contracts.rs` / `tests/sdd_spec.rs` / `tests/aidd_boundary.rs`）
  与合成夹具、`benches/hot_path.rs` 微基准。

### Notes

- 本版本**不构成任何生产授权**：`production_decision = NO-GO`，无 Owner 签核文件。
- 夹具全部为合成样本，不是真实源数据，不构成证据。
- 源定义模型不含限流字段：`rate_limit_rps = <数字>` 一律原子失败（授权前不写限流数字）。
