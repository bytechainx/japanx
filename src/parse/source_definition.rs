//! TOML 驱动的源定义解析。
//!
//! 清单 §1.2/§3 的源形态是 `kind=csv_download` + `Shift-JIS → UTF-8` + 礼貌节奏。
//! 本模块把该形态落成**离线契约**：
//!
//! - 输入是 TOML **文本**，不含 URL、凭据或网络参数
//! - 模型只承载清单已钉死的字段（`kind` / `encoding` / `unit` / `scale` /
//!   `metric_codes`）；其余字段一律**原子拒绝**——这使「授权前不写具体限流数字」
//!   成为机器可验的约束：任何 `rate_limit_rps = <数字>` 都会因未知字段被拒绝
//! - **不实装任何编码库**：本层收到的文本已由采集侧转码为 UTF-8；`encoding`
//!   字段只是源侧声明的事实

use serde::Deserialize;

use crate::error::{JapanCbError, JapanCbResult};
use crate::value::{validate_metric_code, JapanCbMetricCode, Unit};

/// 清单 §1.2 声明的源形态。
pub const SOURCE_KIND_CSV_DOWNLOAD: &str = "csv_download";

/// 清单 §1.2/§3 声明的**源侧**编码。
pub const SOURCE_ENCODING_SHIFT_JIS: &str = "shift_jis";

/// 本层解析入口要求的**输入**编码（采集侧转码后的形态）。
pub const PARSER_INPUT_ENCODING: &str = "utf-8";

/// 源定义（TOML 驱动的离线形态）。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct JapanCbSourceDefinition {
    /// 源形态（清单声明为 `csv_download`）。
    kind: String,
    /// 源侧编码声明（清单声明为 `shift_jis`）。
    encoding: String,
    /// 源侧单位。
    unit: Unit,
    /// 源侧量纲缩放。
    scale: f64,
    /// 该源覆盖的 metric_code（必须全部属于清单声明的 12 个）。
    metric_codes: Vec<JapanCbMetricCode>,
}

impl JapanCbSourceDefinition {
    /// 源形态。
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// 源侧编码声明。
    #[must_use]
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// 源侧单位。
    #[must_use]
    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    /// 源侧量纲缩放。
    #[must_use]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// 该源覆盖的 metric_code。
    #[must_use]
    pub fn metric_codes(&self) -> &[JapanCbMetricCode] {
        &self.metric_codes
    }
}

/// TOML 输入形态。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSourceDefinition {
    /// 源形态。
    kind: String,
    /// 源侧编码声明。
    encoding: String,
    /// 源侧单位。
    unit: String,
    /// 源侧量纲缩放。
    scale: f64,
    /// 该源覆盖的 metric_code。
    metric_codes: Vec<String>,
}

/// 解析并校验源定义。
///
/// 校验内容：`kind` 必须是 [`SOURCE_KIND_CSV_DOWNLOAD`]、`encoding` 必须是
/// [`SOURCE_ENCODING_SHIFT_JIS`]、`unit` 非空、`scale` 有限且非 0、
/// `metric_codes` 非空且互不重复、且每个都必须是清单声明的 12 个之一。
///
/// 未知字段（**含 `rate_limit_rps`**）一律原子失败。
///
/// # Examples
///
/// ```
/// use japanx::{parse_japanx_source_definition, SOURCE_KIND_CSV_DOWNLOAD};
///
/// # fn main() -> Result<(), japanx::JapanCbError> {
/// let toml = r#"
/// kind = "csv_download"
/// encoding = "shift_jis"
/// unit = "100 million yen"
/// scale = 1
/// metric_codes = ["JP.BS.TOTAL_ASSETS"]
/// "#;
/// let definition = parse_japanx_source_definition(toml)?;
/// assert_eq!(definition.kind(), SOURCE_KIND_CSV_DOWNLOAD);
/// assert_eq!(definition.metric_codes().len(), 1);
/// # Ok(())
/// # }
/// ```
pub fn parse_japanx_source_definition(input: &str) -> JapanCbResult<JapanCbSourceDefinition> {
    let wire: WireSourceDefinition = toml::from_str(input).map_err(describe_toml_error)?;

    if wire.kind != SOURCE_KIND_CSV_DOWNLOAD {
        return Err(JapanCbError::SemanticallyRejected(format!(
            "清单声明的源形态是 {SOURCE_KIND_CSV_DOWNLOAD}，收到 {}",
            wire.kind
        )));
    }
    if wire.encoding.trim().is_empty() {
        return Err(JapanCbError::Missing("源侧编码声明为空".to_owned()));
    }
    if wire.encoding != SOURCE_ENCODING_SHIFT_JIS {
        return Err(JapanCbError::SemanticallyRejected(format!(
            "清单声明的源侧编码是 {SOURCE_ENCODING_SHIFT_JIS}，收到 {}",
            wire.encoding
        )));
    }
    let unit = Unit::try_new(&wire.unit)?;
    if !wire.scale.is_finite() || wire.scale == 0.0 {
        return Err(JapanCbError::Invalid(format!(
            "scale 必须是有限非零数，收到 {}",
            wire.scale
        )));
    }
    if wire.metric_codes.is_empty() {
        return Err(JapanCbError::Missing("metric_codes 为空".to_owned()));
    }
    let mut metric_codes: Vec<JapanCbMetricCode> = Vec::with_capacity(wire.metric_codes.len());
    for code in &wire.metric_codes {
        let parsed = validate_metric_code(code)?;
        if !parsed.is_declared() {
            return Err(JapanCbError::SemanticallyRejected(format!(
                "清单未声明该 metric_code：{code}"
            )));
        }
        if metric_codes
            .iter()
            .any(|existing| existing.as_str() == parsed.as_str())
        {
            return Err(JapanCbError::SemanticallyRejected(format!(
                "metric_codes 中重复声明：{code}"
            )));
        }
        metric_codes.push(parsed);
    }

    Ok(JapanCbSourceDefinition {
        kind: wire.kind,
        encoding: wire.encoding,
        unit,
        scale: wire.scale,
        metric_codes,
    })
}

/// 把 TOML 解析错误转成本层错误。
///
/// **只带出字节区间，不回显消息与源码片段**：`toml` 的默认消息会把非法取值乃至
/// 整行源码拼进字符串。
fn describe_toml_error(error: toml::de::Error) -> JapanCbError {
    let span = error.span().map_or_else(
        || "<未知>".to_owned(),
        |span| format!("{}..{}", span.start, span.end),
    );
    JapanCbError::Invalid(format!(
        "TOML 文档不可解析（字节区间 {span}）：请检查字段名、类型与取值形态"
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_japanx_source_definition, PARSER_INPUT_ENCODING, SOURCE_ENCODING_SHIFT_JIS,
        SOURCE_KIND_CSV_DOWNLOAD,
    };
    use crate::error::JapanCbErrorKind;

    fn valid_toml() -> &'static str {
        r#"
kind = "csv_download"
encoding = "shift_jis"
unit = "100 million yen"
scale = 1
metric_codes = ["JP.BS.TOTAL_ASSETS", "JP.CA.EXCESS_RESERVES"]
"#
    }

    #[test]
    fn parses_the_declared_source_shape() {
        let definition = parse_japanx_source_definition(valid_toml()).expect("合法源定义");
        assert_eq!(definition.kind(), SOURCE_KIND_CSV_DOWNLOAD);
        assert_eq!(definition.encoding(), SOURCE_ENCODING_SHIFT_JIS);
        assert_eq!(definition.unit().text(), "100 million yen");
        assert_eq!(definition.scale(), 1.0);
        assert_eq!(definition.metric_codes().len(), 2);
        assert_eq!(PARSER_INPUT_ENCODING, "utf-8");
    }

    #[test]
    fn rate_limit_numbers_are_rejected_atomically() {
        // 「授权前不写具体限流数字」是本库的机器可验约束：任何限流数字都因未知字段失败。
        for extra in [
            "rate_limit_rps = 5\n",
            "rate_limit_rps = 0.5\n",
            "rate_limit = 5\n",
        ] {
            let document = format!("{extra}{}", valid_toml());
            let error =
                parse_japanx_source_definition(&document).expect_err("限流数字必须被原子拒绝");
            assert_eq!(error.kind(), JapanCbErrorKind::Invalid, "{extra}");
        }
    }

    #[test]
    fn unknown_fields_are_rejected_atomically() {
        for extra in [
            "endpoint = \"x\"\n",
            "url = \"x\"\n",
            "schedule = \"09:15\"\n",
        ] {
            let document = format!("{extra}{}", valid_toml());
            assert!(
                parse_japanx_source_definition(&document).is_err(),
                "{extra}"
            );
        }
    }

    #[test]
    fn toml_error_message_does_not_echo_the_source_line() {
        let document = format!(
            "{}\nunknown_field = \"synthetic-secret-value\"\n",
            valid_toml()
        );
        let error = parse_japanx_source_definition(&document).expect_err("未知字段必须失败");
        let message = error.to_string();
        assert!(!message.contains("synthetic-secret-value"), "{message}");
        assert!(!message.contains("unknown_field"), "{message}");
    }

    #[test]
    fn kind_and_encoding_must_match_the_manifest() {
        let wrong_kind = valid_toml().replace("csv_download", "json_api");
        assert_eq!(
            parse_japanx_source_definition(&wrong_kind)
                .expect_err("源形态必须是 csv_download")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );

        let wrong_encoding = valid_toml().replace("shift_jis", "utf-8");
        assert_eq!(
            parse_japanx_source_definition(&wrong_encoding)
                .expect_err("源侧编码必须是 shift_jis")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }

    #[test]
    fn undeclared_metric_codes_are_rejected() {
        let undeclared = valid_toml().replace(
            "[\"JP.BS.TOTAL_ASSETS\", \"JP.CA.EXCESS_RESERVES\"]",
            "[\"JP.TK.LARGE_NON_MFG_DI\"]",
        );
        assert_eq!(
            parse_japanx_source_definition(&undeclared)
                .expect_err("清单未声明的 metric_code 必须拒绝")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }

    #[test]
    fn empty_or_invalid_fields_are_rejected() {
        let empty_codes =
            valid_toml().replace("[\"JP.BS.TOTAL_ASSETS\", \"JP.CA.EXCESS_RESERVES\"]", "[]");
        assert_eq!(
            parse_japanx_source_definition(&empty_codes)
                .expect_err("空 metric_codes 必须拒绝")
                .kind(),
            JapanCbErrorKind::Missing
        );

        let zero_scale = valid_toml().replace("scale = 1", "scale = 0");
        assert_eq!(
            parse_japanx_source_definition(&zero_scale)
                .expect_err("scale 不得为 0")
                .kind(),
            JapanCbErrorKind::Invalid
        );

        let blank_unit = valid_toml().replace("unit = \"100 million yen\"", "unit = \"  \"");
        assert_eq!(
            parse_japanx_source_definition(&blank_unit)
                .expect_err("空单位必须拒绝")
                .kind(),
            JapanCbErrorKind::Missing
        );

        let duplicate = valid_toml().replace(
            "[\"JP.BS.TOTAL_ASSETS\", \"JP.CA.EXCESS_RESERVES\"]",
            "[\"JP.MS.M2\", \"JP.MS.M2\"]",
        );
        assert_eq!(
            parse_japanx_source_definition(&duplicate)
                .expect_err("重复 metric_code 必须拒绝")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }
}
