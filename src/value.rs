//! japanx 值对象门面。
//!
//! 子模块按职责拆分（日期 / 期间与频率 / metric_code / 数据点），依赖方向单向：
//! `error ← value.*`，各子模块之间不构成环。

mod datapoint;
mod date;
mod metric;
mod period;

pub use datapoint::{
    ensure_no_converted_or_derived_field, validate_data_point, DataPointValue, JapanCbDataPoint,
    JapanCbMissingReason, Revision, Unit, SOURCE_UNIT_HUNDRED_MILLION_YEN,
};
pub use date::Date;
pub use metric::{
    claim_curve_authority, claim_forwarded_series_sovereignty, is_declared_metric_code,
    reject_excluded_fx_symbol, reject_silent_fx_substitution, validate_metric_code,
    JapanCbMetricCode, MetricCategory, DECLARED_METRIC_CODES, EXCLUDED_FX_SYMBOLS,
    FRED_EXCLUSIVE_FX_SERIES, JP_BS_ETF_HOLDINGS, JP_BS_JGB_HOLDINGS, JP_BS_TOTAL_ASSETS,
    JP_CA_EXCESS_RESERVES, JP_CA_TOTAL, JP_FX_INTERVENTION, JP_IR_POLICY_RATE, JP_IR_TONA,
    JP_MB_TOTAL, JP_MO_JGB_PURCHASE, JP_MS_M2, JP_TK_LARGE_MFG_DI,
};
pub use period::{Frequency, Period};

#[cfg(test)]
mod tests {
    use super::{
        validate_metric_code, DataPointValue, Frequency, JapanCbDataPoint, Period, Unit,
        DECLARED_METRIC_CODES, JP_MB_TOTAL, SOURCE_UNIT_HUNDRED_MILLION_YEN,
    };
    use crate::error::JapanCbErrorKind;

    #[test]
    fn facade_reexports_are_usable_together() {
        let point = JapanCbDataPoint::new(
            validate_metric_code(JP_MB_TOTAL).expect("合法 metric_code"),
            Period::year(2026).expect("合法期间"),
            Frequency::Annual,
            DataPointValue::Value(1.0),
            Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN).expect("合法单位"),
            None,
        )
        .expect("合法数据点");
        assert_eq!(point.data_point_id(), "JP.MB.TOTAL:2026:none");
        assert_eq!(DECLARED_METRIC_CODES.len(), 12);
        assert_eq!(
            validate_metric_code("XX").expect_err("非法").kind(),
            JapanCbErrorKind::Invalid
        );
    }
}
