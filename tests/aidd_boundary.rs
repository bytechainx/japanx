#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! AIDD 对抗 / 边界用例（特性 005）。
//!
//! 候选由 AI 生成，逐条人工复核后仅保留「结论=保留」项；丢弃项登记于 PR 描述。
//!
//! // AIDD: metric_code 前缀非 JP / 为空 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §1 规范形首段须为 JP | 结论=保留
//! // AIDD: 旬序号取 0 与 4 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 旬范围 1–3 | 结论=保留
//! // AIDD: 世纪闰年 1900-02-29 / 2000-02-29 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 严格日期解析 | 结论=保留
//! // AIDD: 超长 metric_code（10000 字符指标段） | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §1 规范形不得 panic | 结论=保留
//! // AIDD: metric_code 恰好 4 段（含子维度） | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §1 规范形允许子维度 | 结论=保留
//! // AIDD: 非 ASCII 单位与 metric_code 段 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §3 单位原样保留 / §1 段须 ASCII | 结论=保留
//! // AIDD: 同身份不同 revision 是否算重复 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §6 身份含修订标识 | 结论=保留
//! // AIDD: 极大 / 极小 / 负零 f64 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §3 值原样保留 | 结论=保留
//! // AIDD: _synthetic=false 的输入 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §6 只接受合成标注输入 | 结论=保留

use japanx::{
    parse_japanx_data_points, validate_metric_code, DataPointValue, Date, Frequency,
    JapanCbDataPoint, JapanCbErrorKind, Period, Revision, Unit, SOURCE_UNIT_HUNDRED_MILLION_YEN,
};

fn unit() -> Unit {
    Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN).expect("合法单位")
}

fn point(period: Period, frequency: Frequency, value: DataPointValue) -> JapanCbDataPoint {
    JapanCbDataPoint::new(
        validate_metric_code("JP.MS.M2").expect("合法 metric_code"),
        period,
        frequency,
        value,
        unit(),
        None,
    )
    .expect("合法数据点")
}

/// 边界：metric_code 首段必须逐字为 `JP`。
#[test]
fn aidd_metric_code_prefix_boundary() {
    for code in ["", "JP", "jp.MS.M2", "JPX.MS.M2", "US.MS.M2", ".MS.M2"] {
        let error = validate_metric_code(code).expect_err(code);
        assert_eq!(error.kind(), JapanCbErrorKind::Invalid, "{code:?}");
    }
}

/// 边界：旬序号 0 与 4 必须拒绝，1–3 合法。
#[test]
fn aidd_ten_day_segment_bounds() {
    assert!(Period::ten_day(2026, 9, 0).is_err());
    assert!(Period::ten_day(2026, 9, 4).is_err());
    assert!(Period::ten_day(2026, 9, 1).is_ok());
    assert!(Period::ten_day(2026, 9, 3).is_ok());
    assert!(Period::parse("2026-09-T0").is_err());
    assert!(Period::parse("2026-09-T9").is_err());
}

/// 边界：世纪闰年规则。
#[test]
fn aidd_century_leap_year_boundary() {
    assert!(Date::new(1900, 2, 29).is_err());
    assert!(Date::new(2000, 2, 29).is_ok());
    assert!(Date::new(2100, 2, 29).is_err());
    assert!(Date::new(2024, 2, 29).is_ok());
}

/// 边界：超长 metric_code 不得 panic，且仍属规范形。
#[test]
fn aidd_extremely_long_metric_code() {
    let long = format!("JP.MS.{}", "A".repeat(10_000));
    let parsed = validate_metric_code(&long).expect("超长指标段仍是规范形");
    assert_eq!(parsed.as_str().len(), 10_006);
    assert!(!parsed.is_declared(), "超长码不在清单声明集内");
}

/// 边界：4 段（含子维度）合法，5 段（含子维度的子维度）非法。
#[test]
fn aidd_sub_dimension_arity() {
    assert!(validate_metric_code("JP.BS.TOTAL_ASSETS.SUB").is_ok());
    assert!(validate_metric_code("JP.BS.TOTAL_ASSETS.SUB.MORE").is_err());
}

/// 边界：非 ASCII 段必须拒绝，非 ASCII 单位必须原样保留。
#[test]
fn aidd_non_ascii_boundary() {
    let error = validate_metric_code("JP.BS.指標").expect_err("非 ASCII 段必须拒绝");
    assert_eq!(error.kind(), JapanCbErrorKind::SemanticallyRejected);

    let unit = Unit::try_new("  億円 (100 million yen)  ").expect("合法单位");
    assert_eq!(unit.text(), "億円 (100 million yen)");
}

/// 边界：身份含修订标识，故「同 metric_code 同 period、不同 revision」不是重复。
#[test]
fn aidd_same_identity_different_revision_is_not_duplicate() {
    let document = r#"{ "_synthetic": true, "_note": "合成样本",
        "data_points": [
          { "metric_code": "JP.MS.M2", "period": "2026-08", "frequency": "Monthly",
            "value": 1.0, "unit": "100 million yen" },
          { "metric_code": "JP.MS.M2", "period": "2026-08", "frequency": "Monthly",
            "value": 1.0, "unit": "100 million yen", "revision": "SYNTH_REV" }
        ] }"#;
    let points = parse_japanx_data_points(document).expect("不同 revision 不算重复");
    assert_eq!(points.len(), 2);
    assert_ne!(points[0].data_point_id(), points[1].data_point_id());
}

/// 边界：极大 / 极小 / 负零 f64 原样保留。
#[test]
fn aidd_extreme_f64_values_are_preserved() {
    for value in [f64::MAX, f64::MIN, 0.0, -0.0] {
        let point = point(
            Period::month(2026, 8).expect("合法期间"),
            Frequency::Monthly,
            DataPointValue::Value(value),
        );
        assert_eq!(point.value.value(), Some(value));
    }
}

/// 边界：revision 只承载原文，不做形态推断（清单未定义其形态）。
#[test]
fn aidd_revision_is_opaque_text() {
    for text in ["R1", "2026-09-20", "SYNTH_REV", "1"] {
        let revision = Revision::try_new(text).expect("任意非空原文");
        assert_eq!(revision.text(), text);
    }
    assert!(Revision::try_new("\t\n").is_err());
}

/// 边界：显式声明非合成的输入必须被拒绝。
#[test]
fn aidd_non_synthetic_input_is_refused() {
    let document = r#"{ "_synthetic": false, "_note": "x", "data_points": [] }"#;
    assert_eq!(
        parse_japanx_data_points(document)
            .expect_err("不得把非合成输入当源数据")
            .kind(),
        JapanCbErrorKind::SemanticallyRejected
    );
}

/// 边界：旬期间与旬频率必须同时给出，否则粒度自洽校验拒绝。
#[test]
fn aidd_ten_day_period_requires_matching_frequency() {
    let matched = point(
        Period::ten_day(2026, 9, 2).expect("合法旬期间"),
        Frequency::TenDay,
        DataPointValue::Value(1.0),
    );
    assert_eq!(matched.data_point_id(), "JP.MS.M2:2026-09-T2:none");

    let mismatched = JapanCbDataPoint::new(
        validate_metric_code("JP.MS.M2").expect("合法"),
        Period::ten_day(2026, 9, 2).expect("合法旬期间"),
        Frequency::Monthly,
        DataPointValue::Value(1.0),
        unit(),
        None,
    );
    assert_eq!(
        mismatched.expect_err("粒度不一致必须拒绝").kind(),
        JapanCbErrorKind::SemanticallyRejected
    );
}
