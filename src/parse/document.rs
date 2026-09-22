//! 数据点的离线 JSON 解析。
//!
//! 输入形态：`{ "_synthetic": <bool>, "_note": <string>, "data_points": [ … ] }`。
//! 字段名取自清单 §1.1（`metric_code`）与 §4（`obs_time` / `Period` / `value` /
//! `revision` / `unit`），其中清单 §4 的 `Period{Daily|TenDay|Monthly|Quarterly|Annual}`
//! 在本库落为 `frequency`（本库自有拼写），期间身份落为 `period` 键。
//!
//! 两条硬约束：**未知字段原子失败**、**重复身份拒绝**（不去重）。

use serde::Deserialize;

use crate::error::{JapanCbError, JapanCbResult};
use crate::value::{
    validate_metric_code, DataPointValue, Frequency, JapanCbDataPoint, JapanCbMissingReason,
    Period, Revision, Unit,
};

/// 顶层文档。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDocument {
    /// 合成样本标记。**必须显式存在且为 `true`**。
    #[serde(rename = "_synthetic")]
    synthetic: bool,
    /// 合成样本说明。必须非空。
    #[serde(rename = "_note")]
    note: String,
    /// 数据点数组。
    data_points: Vec<WireDataPoint>,
}

/// 单个数据点的输入形态。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDataPoint {
    /// 清单 §1.1 的 `JP.*` metric_code。
    metric_code: String,
    /// 期间（清单 §4 的 `obs_time`）。
    period: String,
    /// 频率（清单 §4 的 `Period{Y}` 取值，逐字为 `Daily|TenDay|Monthly|Quarterly|Annual`）。
    frequency: String,
    /// 数值；`null` 表示缺失。
    value: Option<f64>,
    /// 源侧状态原文（`value` 为 `null` 时参与表达缺失原因）。
    #[serde(default)]
    obs_status: Option<String>,
    /// 源侧单位原文。
    unit: String,
    /// 源侧修订标识原文。
    #[serde(default)]
    revision: Option<String>,
}

/// 解析一整个离线文档。
///
/// 返回的数据点顺序与文档一致；任一条失败即**整体失败**（不返回部分结果）。
///
/// 输入必须显式标注 `_synthetic = true` 且给出非空 `_note`：本层只处理合成样本，
/// 拒绝把未标注输入当作源数据（对应 `FR-057` 的证据纪律）。
///
/// # Examples
///
/// ```
/// use japanx::parse_japanx_data_points;
///
/// # fn main() -> Result<(), japanx::JapanCbError> {
/// let document = r#"{ "_synthetic": true, "_note": "合成样本", "data_points": [] }"#;
/// let points = parse_japanx_data_points(document)?;
/// assert!(points.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn parse_japanx_data_points(input: &str) -> JapanCbResult<Vec<JapanCbDataPoint>> {
    let WireDocument {
        synthetic,
        note,
        data_points: wire_points,
    } = serde_json::from_str(input).map_err(describe_json_error)?;

    if !synthetic {
        return Err(JapanCbError::SemanticallyRejected(
            "离线解析只接受显式标注 `_synthetic = true` 的合成样本：本层不得把未标注输入当作源数据"
                .to_owned(),
        ));
    }
    if note.trim().is_empty() {
        return Err(JapanCbError::Missing(
            "`_note` 为空：合成样本必须说明「不是真实源数据」".to_owned(),
        ));
    }

    let mut points = Vec::with_capacity(wire_points.len());
    let mut seen: Vec<String> = Vec::with_capacity(wire_points.len());
    for (index, wire) in wire_points.into_iter().enumerate() {
        let point = convert(wire).map_err(|error| annotate(index, error))?;
        let identity = point.data_point_id();
        if seen.contains(&identity) {
            return Err(JapanCbError::SemanticallyRejected(format!(
                "第 {index} 条数据点身份重复：{identity}（本库选择拒绝而非去重）"
            )));
        }
        seen.push(identity);
        points.push(point);
    }
    Ok(points)
}

/// 单条数据点的形态转换。
fn convert(wire: WireDataPoint) -> JapanCbResult<JapanCbDataPoint> {
    let metric_code = validate_metric_code(&wire.metric_code)?;
    let period = Period::parse(&wire.period)?;
    let frequency = Frequency::from_source_token(&wire.frequency)?;
    let unit = Unit::try_new(&wire.unit)?;
    let revision = wire
        .revision
        .as_deref()
        .map(Revision::try_new)
        .transpose()?;
    let value = match wire.value {
        Some(value) => DataPointValue::Value(value),
        None => DataPointValue::Missing(match wire.obs_status.as_deref() {
            Some(status) if !status.trim().is_empty() => {
                JapanCbMissingReason::SourceStatus(status.to_owned())
            }
            _ => JapanCbMissingReason::Unspecified,
        }),
    };
    JapanCbDataPoint::new(metric_code, period, frequency, value, unit, revision)
}

/// 把 serde_json 的解析错误转成本层错误。
///
/// **只带出行号、列号与错误分类，不回显原文片段**——serde 的默认消息会把非法取值
/// 原样拼进字符串，那样会把凭据或整行配置源码带进错误消息。
fn describe_json_error(error: serde_json::Error) -> JapanCbError {
    JapanCbError::Invalid(format!(
        "JSON 文档不可解析（行 {} 列 {}，分类 {:?}）：请检查字段名、类型与取值形态",
        error.line(),
        error.column(),
        error.classify()
    ))
}

/// 给单条数据点的错误补上位置信息，保持原分类不变。
fn annotate(index: usize, error: JapanCbError) -> JapanCbError {
    let prefix = format!("第 {index} 条数据点：");
    match error {
        JapanCbError::Invalid(message) => JapanCbError::Invalid(format!("{prefix}{message}")),
        JapanCbError::Missing(message) => JapanCbError::Missing(format!("{prefix}{message}")),
        JapanCbError::AuthorizationDenied(message) => {
            JapanCbError::AuthorizationDenied(format!("{prefix}{message}"))
        }
        JapanCbError::RoutedElsewhere(message) => {
            JapanCbError::RoutedElsewhere(format!("{prefix}{message}"))
        }
        JapanCbError::WriteAuthorityDenied(message) => {
            JapanCbError::WriteAuthorityDenied(format!("{prefix}{message}"))
        }
        JapanCbError::SemanticallyRejected(message) => {
            JapanCbError::SemanticallyRejected(format!("{prefix}{message}"))
        }
        JapanCbError::NotApplicable(message) => {
            JapanCbError::NotApplicable(format!("{prefix}{message}"))
        }
        JapanCbError::Invariant(message) => JapanCbError::Invariant(format!("{prefix}{message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_japanx_data_points;
    use crate::error::JapanCbErrorKind;
    use crate::value::DECLARED_METRIC_CODES;

    const NOTE: &str = "本文件为特性 005 自拟的合成样本，不是真实源数据，不构成任何证据。";

    fn wrap(data_points: &str) -> String {
        format!(r#"{{ "_synthetic": true, "_note": "{NOTE}", "data_points": [ {data_points} ] }}"#)
    }

    fn point(metric_code: &str, period: &str, frequency: &str, extra: &str) -> String {
        format!(
            r#"{{ "metric_code": "{metric_code}", "period": "{period}", "frequency": "{frequency}", "value": 1.0, "unit": "100 million yen"{extra} }}"#
        )
    }

    #[test]
    fn parses_metric_codes_periods_and_units() {
        let document = wrap(&format!(
            "{}, {}",
            point("JP.BS.TOTAL_ASSETS", "2026-08", "Monthly", ""),
            point("JP.IR.TONA", "2026-09-18", "Daily", "")
        ));
        let points = parse_japanx_data_points(&document).expect("合成文档应可解析");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].metric_code.as_str(), "JP.BS.TOTAL_ASSETS");
        assert!(points[0].metric_code.is_declared());
        assert_eq!(points[0].unit.text(), "100 million yen");
        assert_eq!(points[0].revision, None);
        assert_eq!(points[1].period.key(), "2026-09-18");
    }

    #[test]
    fn ten_day_period_and_frequency_are_supported() {
        let document = wrap(&point("JP.CA.EXCESS_RESERVES", "2026-09-T2", "TenDay", ""));
        let points = parse_japanx_data_points(&document).expect("旬期间应可解析");
        assert_eq!(points[0].period.key(), "2026-09-T2");
    }

    #[test]
    fn unmarked_input_is_refused() {
        assert!(parse_japanx_data_points(r#"{ "data_points": [] }"#).is_err());

        let contradicted = r#"{ "_synthetic": false, "_note": "x", "data_points": [] }"#;
        assert_eq!(
            parse_japanx_data_points(contradicted)
                .expect_err("不得把非合成输入当源数据")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );

        let blank_note = r#"{ "_synthetic": true, "_note": "  ", "data_points": [] }"#;
        assert_eq!(
            parse_japanx_data_points(blank_note)
                .expect_err("空说明必须拒绝")
                .kind(),
            JapanCbErrorKind::Missing
        );
    }

    #[test]
    fn unknown_field_fails_atomically() {
        let observation_level = wrap(&point(
            "JP.MS.M2",
            "2026-08",
            "Monthly",
            ", \"value_usd\": 1.0",
        ));
        let error = parse_japanx_data_points(&observation_level).expect_err("未知字段必须失败");
        assert_eq!(error.kind(), JapanCbErrorKind::Invalid);

        let top_level = format!(
            r#"{{ "_synthetic": true, "_note": "{NOTE}", "data_points": [], "stat_code": "x" }}"#
        );
        assert!(parse_japanx_data_points(&top_level).is_err());
    }

    #[test]
    fn missing_field_fails_atomically() {
        let document =
            wrap(r#"{ "metric_code": "JP.MS.M2", "period": "2026-08", "frequency": "Monthly" }"#);
        assert_eq!(
            parse_japanx_data_points(&document)
                .expect_err("缺 unit 必须失败")
                .kind(),
            JapanCbErrorKind::Invalid
        );
    }

    #[test]
    fn json_error_message_does_not_echo_the_offending_value() {
        let document = format!(
            r#"{{ "_synthetic": true, "_note": "{NOTE}", "data_points": [ {{ "metric_code": "synthetic-secret-value", "period": 1 }} ] }}"#
        );
        let error = parse_japanx_data_points(&document).expect_err("非法类型必须失败");
        let message = error.to_string();
        assert!(!message.contains("synthetic-secret-value"), "{message}");
        assert_eq!(error.kind(), JapanCbErrorKind::Invalid);
    }

    #[test]
    fn undeclared_metric_code_is_semantically_rejected_at_declaration_layer() {
        // 规范形合法但不在清单 12 个之内：解析本身通过，声明判定为假。
        let document = wrap(&point("JP.TK.LARGE_NON_MFG_DI", "2026-08", "Monthly", ""));
        let points = parse_japanx_data_points(&document).expect("规范形合法");
        assert!(!points[0].metric_code.is_declared());
        assert!(!DECLARED_METRIC_CODES.contains(&points[0].metric_code.as_str()));
    }

    #[test]
    fn undeclared_frequency_token_is_rejected() {
        for frequency in ["Weekly", "Event", "Irregular", "Tenday"] {
            let document = wrap(&point("JP.MS.M2", "2026-08", frequency, ""));
            let error =
                parse_japanx_data_points(&document).expect_err("清单未声明的期间取值必须拒绝");
            assert_eq!(
                error.kind(),
                JapanCbErrorKind::SemanticallyRejected,
                "{frequency}"
            );
        }
    }

    #[test]
    fn illegal_period_is_rejected() {
        for period in ["2026-2-3", "2026/09/18", "2026-02-30", "2026-13", "2026-Q5"] {
            let document = wrap(&point("JP.MS.M2", period, "Monthly", ""));
            assert!(parse_japanx_data_points(&document).is_err(), "{period}");
        }
    }

    #[test]
    fn duplicate_identity_is_rejected_not_deduplicated() {
        let one = point("JP.MS.M2", "2026-08", "Monthly", "");
        let document = wrap(&format!("{one}, {one}"));
        assert_eq!(
            parse_japanx_data_points(&document)
                .expect_err("重复身份必须拒绝")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }
}
