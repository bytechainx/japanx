//! `JP.*` metric_code：声明集、规范形校验与跨源守卫。
//!
//! 规范形（清单 §1.1）：`JP.<类别>.<指标>[.<子维度>]`。
//!
//! **三处硬禁**都在本文件落成守卫：
//!
//! 1. 曲线归 `yieldx`（#13–14）——[`claim_curve_authority`] 恒拒绝
//! 2. `JPY=X` / `USDJPY=X` 归 `fredx` 独占（`DEXJPUS`）——[`reject_excluded_fx_symbol`] 恒拒绝
//! 3. `JPY=X` / `USDJPY=X` ≠ `DEXJPUS`（近义非同 ID）——[`reject_silent_fx_substitution`]

use std::fmt;

use crate::error::{JapanCbError, JapanCbResult};

/// `JP.BS.TOTAL_ASSETS`：BOJ 总资产。
pub const JP_BS_TOTAL_ASSETS: &str = "JP.BS.TOTAL_ASSETS";
/// `JP.BS.JGB_HOLDINGS`：持有国债。
pub const JP_BS_JGB_HOLDINGS: &str = "JP.BS.JGB_HOLDINGS";
/// `JP.BS.ETF_HOLDINGS`：持有 ETF。
pub const JP_BS_ETF_HOLDINGS: &str = "JP.BS.ETF_HOLDINGS";
/// `JP.IR.POLICY_RATE`：政策利率。
pub const JP_IR_POLICY_RATE: &str = "JP.IR.POLICY_RATE";
/// `JP.IR.TONA`：无担保隔夜。
pub const JP_IR_TONA: &str = "JP.IR.TONA";
/// `JP.MB.TOTAL`：基础货币。
pub const JP_MB_TOTAL: &str = "JP.MB.TOTAL";
/// `JP.MS.M2`：M2。
pub const JP_MS_M2: &str = "JP.MS.M2";
/// `JP.CA.TOTAL`：经常账户余额。
pub const JP_CA_TOTAL: &str = "JP.CA.TOTAL";
/// `JP.CA.EXCESS_RESERVES`：超额准备金。
pub const JP_CA_EXCESS_RESERVES: &str = "JP.CA.EXCESS_RESERVES";
/// `JP.MO.JGB_PURCHASE`：国债购买。
pub const JP_MO_JGB_PURCHASE: &str = "JP.MO.JGB_PURCHASE";
/// `JP.FX.INTERVENTION`：外汇干预。
pub const JP_FX_INTERVENTION: &str = "JP.FX.INTERVENTION";
/// `JP.TK.LARGE_MFG_DI`：短观大型制造业 DI。
pub const JP_TK_LARGE_MFG_DI: &str = "JP.TK.LARGE_MFG_DI";

/// 清单 §1.1 逐字列出的 12 个 metric_code，顺序与清单一致。
pub const DECLARED_METRIC_CODES: [&str; 12] = [
    JP_BS_TOTAL_ASSETS,
    JP_BS_JGB_HOLDINGS,
    JP_BS_ETF_HOLDINGS,
    JP_IR_POLICY_RATE,
    JP_IR_TONA,
    JP_MB_TOTAL,
    JP_MS_M2,
    JP_CA_TOTAL,
    JP_CA_EXCESS_RESERVES,
    JP_MO_JGB_PURCHASE,
    JP_FX_INTERVENTION,
    JP_TK_LARGE_MFG_DI,
];

/// 清单里出现的 `JP.*` 类别码（由 12 个 metric_code 的第三段推得）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricCategory {
    /// `BS`：资产负债表。
    BalanceSheet,
    /// `IR`：利率。
    InterestRate,
    /// `MB`：基础货币。
    MonetaryBase,
    /// `MS`：货币存量。
    MoneyStock,
    /// `CA`：经常账户 / 准备金。
    CurrentAccount,
    /// `MO`：市场操作。
    MarketOperations,
    /// `FX`：外汇。
    ForeignExchange,
    /// `TK`：短观。
    Tankan,
}

impl MetricCategory {
    /// 全部类别（由 12 个 metric_code 出现过的类别穷举，顺序按首次出现）。
    pub const ALL: [Self; 8] = [
        Self::BalanceSheet,
        Self::InterestRate,
        Self::MonetaryBase,
        Self::MoneyStock,
        Self::CurrentAccount,
        Self::MarketOperations,
        Self::ForeignExchange,
        Self::Tankan,
    ];

    /// 类别码（清单里的字面段）。
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::BalanceSheet => "BS",
            Self::InterestRate => "IR",
            Self::MonetaryBase => "MB",
            Self::MoneyStock => "MS",
            Self::CurrentAccount => "CA",
            Self::MarketOperations => "MO",
            Self::ForeignExchange => "FX",
            Self::Tankan => "TK",
        }
    }

    /// 按类别码解析。
    pub fn from_code(code: &str) -> JapanCbResult<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.code() == code)
            .ok_or_else(|| {
                JapanCbError::SemanticallyRejected(format!("清单未声明该 metric_code 类别：{code}"))
            })
    }
}

/// 经规范形校验的 `JP.*` metric_code。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JapanCbMetricCode(String);

impl JapanCbMetricCode {
    /// 规范形文本。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 类别段。
    pub fn category(&self) -> JapanCbResult<MetricCategory> {
        let category = self.0.split('.').nth(1).unwrap_or_default();
        MetricCategory::from_code(category)
    }

    /// 是否属于清单 §1.1 声明的 12 个 metric_code。
    #[must_use]
    pub fn is_declared(&self) -> bool {
        is_declared_metric_code(&self.0)
    }
}

impl fmt::Display for JapanCbMetricCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// 某个 metric_code 是否属于清单 §1.1 声明的 12 个。
#[must_use]
pub fn is_declared_metric_code(code: &str) -> bool {
    DECLARED_METRIC_CODES.contains(&code)
}

/// 按清单 §1.1 的规范形 `JP.<类别>.<指标>[.<子维度>]` 校验并归一化。
///
/// 校验内容：恰好 3 或 4 段、首段为 `JP`、第二段为清单声明过的类别码、
/// 其余段非空且为 ASCII 且不含空白。
///
/// **不**声称掌握源侧 code list：本函数只校验规范形，是否属于清单声明的 12 个
/// 由 [`is_declared_metric_code`] 单独判定。
pub fn validate_metric_code(code: &str) -> JapanCbResult<JapanCbMetricCode> {
    let segments: Vec<&str> = code.split('.').collect();
    if !(3..=4).contains(&segments.len()) {
        return Err(JapanCbError::Invalid(format!(
            "metric_code 段数非法：{code}（须为 JP.<类别>.<指标>[.<子维度>]）"
        )));
    }
    if segments[0] != "JP" {
        return Err(JapanCbError::Invalid(format!(
            "metric_code 前缀必须为 JP：{code}"
        )));
    }
    // 第二段必须是清单声明过的类别码。
    MetricCategory::from_code(segments[1])?;
    for (index, segment) in segments.iter().enumerate().skip(2) {
        if segment.is_empty() {
            return Err(JapanCbError::Missing(format!(
                "metric_code 第 {index} 段为空：{code}"
            )));
        }
        if !segment.is_ascii() || segment.chars().any(char::is_whitespace) {
            return Err(JapanCbError::SemanticallyRejected(format!(
                "metric_code 第 {index} 段含非 ASCII 或空白：{code}"
            )));
        }
    }
    Ok(JapanCbMetricCode(code.to_owned()))
}

/// 禁抢主源符号：`JPY=X` / `USDJPY=X` 归 `fredx` 独占（`DEXJPUS`）。
pub const EXCLUDED_FX_SYMBOLS: [&str; 2] = ["JPY=X", "USDJPY=X"];

/// `fredx` 独占的日元汇率序列。
pub const FRED_EXCLUSIVE_FX_SERIES: &str = "DEXJPUS";

/// 拒绝把 `JPY=X` / `USDJPY=X` 主张为本库的主源符号。
///
/// 跨源路由契约 §4 第 22 行 / §5：两者归 `fredx` 独占（`DEXJPUS`），
/// 本库不得主张、不得争抢。
pub fn reject_excluded_fx_symbol(symbol: &str) -> JapanCbResult<()> {
    if EXCLUDED_FX_SYMBOLS.contains(&symbol) {
        return Err(JapanCbError::WriteAuthorityDenied(format!(
            "{symbol} 归 fredx 独占（{FRED_EXCLUSIVE_FX_SERIES}）：本库不得主张主源"
        )));
    }
    Ok(())
}

/// 拒绝近义非同 ID 的静默替换：`JPY=X` / `USDJPY=X` **不等于** `DEXJPUS`。
///
/// 跨源路由契约 §4 第 22 行：左列与右列语义不同，MUST NOT 互为别名、
/// MUST NOT 静默替换。
pub fn reject_silent_fx_substitution(left: &str, right: &str) -> JapanCbResult<()> {
    let pair = [left, right];
    let has_excluded = EXCLUDED_FX_SYMBOLS
        .iter()
        .any(|symbol| pair.contains(symbol));
    if has_excluded && pair.contains(&FRED_EXCLUSIVE_FX_SERIES) {
        return Err(JapanCbError::SemanticallyRejected(format!(
            "{left} 与 {right} 语义不同：前者归 fredx 独占，MUST NOT 互为别名"
        )));
    }
    Ok(())
}

/// 声明 MOF / BOJ 曲线的权威。
///
/// 跨源路由契约 §2 与 §7：MOF / BOJ 曲线归 `yieldx` #13–14，
/// **本库不晋级**，故本函数恒返回 [`JapanCbError::RoutedElsewhere`]。
pub fn claim_curve_authority() -> JapanCbResult<()> {
    Err(JapanCbError::RoutedElsewhere(
        "MOF / BOJ 曲线归 yieldx #13–14：本库不晋级、不复制 kernel".to_owned(),
    ))
}

/// 声明 JGB 10Y / 日本 CPI 的写入主权。
///
/// 跨源路由契约 §3 与名册 `routing`：
/// `jgb10y_cpi_forwarded_by_fredx_sovereignty_pending`——主权 vs 转发**待裁**，
/// 故本函数恒返回 [`JapanCbError::WriteAuthorityDenied`]。
pub fn claim_forwarded_series_sovereignty() -> JapanCbResult<()> {
    Err(JapanCbError::WriteAuthorityDenied(
        "JGB 10Y / 日本 CPI 的主权与转发归属待裁（pending）：本库不得主张权威写入".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        claim_curve_authority, claim_forwarded_series_sovereignty, is_declared_metric_code,
        reject_excluded_fx_symbol, reject_silent_fx_substitution, validate_metric_code,
        MetricCategory, DECLARED_METRIC_CODES, JP_BS_ETF_HOLDINGS, JP_BS_JGB_HOLDINGS,
        JP_BS_TOTAL_ASSETS, JP_CA_EXCESS_RESERVES, JP_CA_TOTAL, JP_FX_INTERVENTION,
        JP_IR_POLICY_RATE, JP_IR_TONA, JP_MB_TOTAL, JP_MO_JGB_PURCHASE, JP_MS_M2,
        JP_TK_LARGE_MFG_DI,
    };
    use crate::error::JapanCbErrorKind;

    #[test]
    fn twelve_metric_codes_are_listed_verbatim() {
        assert_eq!(
            DECLARED_METRIC_CODES,
            [
                "JP.BS.TOTAL_ASSETS",
                "JP.BS.JGB_HOLDINGS",
                "JP.BS.ETF_HOLDINGS",
                "JP.IR.POLICY_RATE",
                "JP.IR.TONA",
                "JP.MB.TOTAL",
                "JP.MS.M2",
                "JP.CA.TOTAL",
                "JP.CA.EXCESS_RESERVES",
                "JP.MO.JGB_PURCHASE",
                "JP.FX.INTERVENTION",
                "JP.TK.LARGE_MFG_DI",
            ]
        );
        // 具名常量与集合一致。
        let named = [
            JP_BS_TOTAL_ASSETS,
            JP_BS_JGB_HOLDINGS,
            JP_BS_ETF_HOLDINGS,
            JP_IR_POLICY_RATE,
            JP_IR_TONA,
            JP_MB_TOTAL,
            JP_MS_M2,
            JP_CA_TOTAL,
            JP_CA_EXCESS_RESERVES,
            JP_MO_JGB_PURCHASE,
            JP_FX_INTERVENTION,
            JP_TK_LARGE_MFG_DI,
        ];
        assert_eq!(named, DECLARED_METRIC_CODES);
    }

    #[test]
    fn declared_codes_pass_shape_validation_and_are_declared() {
        for code in DECLARED_METRIC_CODES {
            let parsed = validate_metric_code(code).expect(code);
            assert_eq!(parsed.as_str(), code);
            assert!(parsed.is_declared(), "{code}");
            assert_eq!(parsed.to_string(), code);
            assert!(parsed.category().is_ok(), "{code}");
        }
    }

    #[test]
    fn undeclared_but_well_formed_code_is_not_declared() {
        // 规范形合法但不在清单 12 个之内：结构上通过、声明上拒绝。
        let parsed = validate_metric_code("JP.TK.LARGE_NON_MFG_DI").expect("规范形合法");
        assert!(!parsed.is_declared());
        assert!(!is_declared_metric_code("JP.TK.LARGE_NON_MFG_DI"));
    }

    #[test]
    fn metric_code_shape_violations_are_rejected() {
        let invalid = [
            "",
            "JP",
            "JP.BS",
            "JP.BS.TOTAL_ASSETS.EXTRA.MORE",
            "US.BS.TOTAL_ASSETS",
            "JP.XX.TOTAL_ASSETS",
            "JP.BS.",
            "JP..TOTAL_ASSETS",
            "JP.BS.TOTAL ASSETS",
            "JP.BS.TOTAL_ASSETS.指標",
        ];
        for code in invalid {
            assert!(validate_metric_code(code).is_err(), "{code:?}");
        }
        // 类别码非法 ⇒ 语义拒绝。
        assert_eq!(
            validate_metric_code("JP.XX.Y")
                .expect_err("类别未声明")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
        // 子维度形态合法。
        assert!(validate_metric_code("JP.BS.TOTAL_ASSETS.SUB").is_ok());
    }

    #[test]
    fn eight_categories_cover_the_declared_codes() {
        let codes: Vec<&str> = MetricCategory::ALL
            .iter()
            .map(MetricCategory::code)
            .collect();
        assert_eq!(codes, ["BS", "IR", "MB", "MS", "CA", "MO", "FX", "TK"]);
        for category in MetricCategory::ALL {
            assert_eq!(
                MetricCategory::from_code(category.code()).expect("可回读"),
                category
            );
        }
        assert!(MetricCategory::from_code("ZZ").is_err());
        // 12 个声明码的类别全部落在 8 类内。
        for code in DECLARED_METRIC_CODES {
            assert!(validate_metric_code(code).expect(code).category().is_ok());
        }
    }

    #[test]
    fn excluded_fx_symbols_are_refused() {
        for symbol in ["JPY=X", "USDJPY=X"] {
            let error = reject_excluded_fx_symbol(symbol).expect_err(symbol);
            assert_eq!(error.kind(), JapanCbErrorKind::WriteAuthorityDenied);
        }
        assert!(reject_excluded_fx_symbol("DEXJPUS").is_ok());
        assert!(reject_excluded_fx_symbol("JP.BS.TOTAL_ASSETS").is_ok());
    }

    #[test]
    fn silent_fx_substitution_is_rejected() {
        for pair in [("JPY=X", "DEXJPUS"), ("DEXJPUS", "USDJPY=X")] {
            let error = reject_silent_fx_substitution(pair.0, pair.1).expect_err("不得互换");
            assert_eq!(error.kind(), JapanCbErrorKind::SemanticallyRejected);
        }
        assert!(reject_silent_fx_substitution("JPY=X", "USDJPY=X").is_ok());
        assert!(reject_silent_fx_substitution("JP.MS.M2", "JP.MB.TOTAL").is_ok());
    }

    #[test]
    fn curve_and_forwarded_sovereignty_are_refused() {
        assert_eq!(
            claim_curve_authority().expect_err("曲线归 yieldx").kind(),
            JapanCbErrorKind::RoutedElsewhere
        );
        assert_eq!(
            claim_forwarded_series_sovereignty()
                .expect_err("主权待裁")
                .kind(),
            JapanCbErrorKind::WriteAuthorityDenied
        );
    }
}
