#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! TDD 行为契约（特性 005）。
//!
//! 入口集合 = 本 crate 全部公开入口（含 `validate*` / 判定函数 / 解析器）。
//!
//! 下表为每个入口**声明**的语义变异与应红 / 应绿用例（用例名均在文件内真实存在）。
//! 其中 **24 条**已在 `/tmp` 隔离副本（独立 `CARGO_TARGET_DIR`）上实测：变异后指定用例
//! 观测为红、原树同名用例观测为绿；实测清单与复现命令见 crate 汇报与 PR 描述。
//! **未实测的行是待执行声明**，不得据其声称已观测。
//!
//! // TDD-PROBE: JapanCbError::kind | 变异：把 Missing 映射为 Invalid | 红=error_kind_maps_every_variant_distinctly | 绿=error_kind_maps_every_variant_distinctly
//! // TDD-PROBE: JapanCbError::is_retryable | 变异：恒返回 true | 红=only_invariant_is_retryable | 绿=only_invariant_is_retryable
//! // TDD-PROBE: DECLARED_METRIC_CODES | 变异：少列一个 metric_code | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_BS_TOTAL_ASSETS | 变异：字面值写成 JP.BS.TOTAL_ASSET | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_BS_JGB_HOLDINGS | 变异：字面值写成 JP.BS.JGB | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_BS_ETF_HOLDINGS | 变异：字面值写成 JP.BS.ETF | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_IR_POLICY_RATE | 变异：字面值写成 JP.IR.POLICY | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_IR_TONA | 变异：字面值写成 JP.IR.TONAR | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_MB_TOTAL | 变异：字面值写成 JP.MB.TOTALS | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_MS_M2 | 变异：字面值写成 JP.MS.M3 | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_CA_TOTAL | 变异：字面值写成 JP.CA.SUM | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_CA_EXCESS_RESERVES | 变异：字面值写成 JP.CA.EXCESS | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_MO_JGB_PURCHASE | 变异：字面值写成 JP.MO.JGB_BUY | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_FX_INTERVENTION | 变异：字面值写成 JP.FX.INTERVENE | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: JP_TK_LARGE_MFG_DI | 变异：字面值写成 JP.TK.MFG_DI | 红=twelve_metric_codes_are_listed_verbatim | 绿=twelve_metric_codes_are_listed_verbatim
//! // TDD-PROBE: validate_metric_code | 变异：接受段数为 2 的输入 | 红=metric_code_shape_is_validated | 绿=metric_code_shape_is_validated
//! // TDD-PROBE: is_declared_metric_code | 变异：对任意规范形返回 true | 红=declared_membership_is_checked_separately | 绿=declared_membership_is_checked_separately
//! // TDD-PROBE: JapanCbMetricCode::as_str | 变异：返回大写化文本 | 红=declared_membership_is_checked_separately | 绿=declared_membership_is_checked_separately
//! // TDD-PROBE: JapanCbMetricCode::category | 变异：取第 1 段而非第 2 段 | 红=eight_categories_are_listed | 绿=eight_categories_are_listed
//! // TDD-PROBE: JapanCbMetricCode::is_declared | 变异：恒返回 true | 红=declared_membership_is_checked_separately | 绿=declared_membership_is_checked_separately
//! // TDD-PROBE: MetricCategory::ALL | 变异：ALL 少列一个类别 | 红=eight_categories_are_listed | 绿=eight_categories_are_listed
//! // TDD-PROBE: MetricCategory::code | 变异：交换 BS 与 IR | 红=eight_categories_are_listed | 绿=eight_categories_are_listed
//! // TDD-PROBE: MetricCategory::from_code | 变异：接受未声明类别 | 红=eight_categories_are_listed | 绿=eight_categories_are_listed
//! // TDD-PROBE: EXCLUDED_FX_SYMBOLS | 变异：只列 JPY=X | 红=excluded_fx_symbols_are_refused | 绿=excluded_fx_symbols_are_refused
//! // TDD-PROBE: FRED_EXCLUSIVE_FX_SERIES | 变异：写成 DEXJPUS2 | 红=excluded_fx_symbols_are_refused | 绿=excluded_fx_symbols_are_refused
//! // TDD-PROBE: reject_excluded_fx_symbol | 变异：对 JPY=X 返回 Ok | 红=excluded_fx_symbols_are_refused | 绿=excluded_fx_symbols_are_refused
//! // TDD-PROBE: reject_silent_fx_substitution | 变异：对 (JPY=X, DEXJPUS) 返回 Ok | 红=silent_fx_substitution_is_rejected | 绿=silent_fx_substitution_is_rejected
//! // TDD-PROBE: claim_curve_authority | 变异：返回 Ok | 红=curve_and_forwarded_sovereignty_are_refused | 绿=curve_and_forwarded_sovereignty_are_refused
//! // TDD-PROBE: claim_forwarded_series_sovereignty | 变异：返回 Ok | 红=curve_and_forwarded_sovereignty_are_refused | 绿=curve_and_forwarded_sovereignty_are_refused
//! // TDD-PROBE: Period::day | 变异：把入参日期换成固定日期 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::ten_day | 变异：接受 segment=4 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::month | 变异：接受 month=13 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::quarter | 变异：接受 quarter=5 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::year | 变异：接受 year=0 | 红=period_constructors_validate_ranges | 绿=period_constructors_validate_ranges
//! // TDD-PROBE: Period::event | 变异：丢弃日期部分 | 红=period_keys_round_trip | 绿=period_keys_round_trip
//! // TDD-PROBE: Period::parse | 变异：把旬键按日解析 | 红=period_keys_round_trip | 绿=period_keys_round_trip
//! // TDD-PROBE: Period::key | 变异：旬输出 2026-09-S2 | 红=period_keys_round_trip | 绿=period_keys_round_trip
//! // TDD-PROBE: Period::frequency | 变异：TenDay 映射为 Daily | 红=period_frequency_covers_declared_granularities | 绿=period_frequency_covers_declared_granularities
//! // TDD-PROBE: Frequency::ALL | 变异：少列 TenDay | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: Frequency::SOURCE_DECLARED | 变异：把 Weekly 也列为清单声明值 | 红=source_tokens_are_limited_to_the_declared_five | 绿=source_tokens_are_limited_to_the_declared_five
//! // TDD-PROBE: Frequency::name | 变异：返回中文名 | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: Frequency::from_name | 变异：大小写不敏感 | 红=frequency_names_round_trip | 绿=frequency_names_round_trip
//! // TDD-PROBE: Frequency::from_source_token | 变异：接受 Weekly | 红=source_tokens_are_limited_to_the_declared_five | 绿=source_tokens_are_limited_to_the_declared_five
//! // TDD-PROBE: Date::new | 变异：不校验闰年 2 月 | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Date::parse | 变异：接受未补零形式 | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Date::to_iso | 变异：月日不补零 | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Date::year/month/day | 变异：交换 month 与 day | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Date::is_leap_year | 变异：忽略百年不闰规则 | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Date::days_in_month | 变异：2 月恒为 28 天 | 红=date_strict_iso_and_leap_years | 绿=date_strict_iso_and_leap_years
//! // TDD-PROBE: Unit::try_new | 变异：允许纯空白单位 | 红=unit_and_revision_keep_source_text | 绿=unit_and_revision_keep_source_text
//! // TDD-PROBE: Unit::text | 变异：返回大写化文本 | 红=unit_and_revision_keep_source_text | 绿=unit_and_revision_keep_source_text
//! // TDD-PROBE: SOURCE_UNIT_HUNDRED_MILLION_YEN | 变异：写成 100 million yen 的错拼 | 红=unit_and_revision_keep_source_text | 绿=unit_and_revision_keep_source_text
//! // TDD-PROBE: Revision::try_new | 变异：允许空修订标识 | 红=unit_and_revision_keep_source_text | 绿=unit_and_revision_keep_source_text
//! // TDD-PROBE: Revision::text | 变异：返回大写化文本 | 红=unit_and_revision_keep_source_text | 绿=unit_and_revision_keep_source_text
//! // TDD-PROBE: JapanCbMissingReason::describe | 变异：Unspecified 返回空串 | 红=missing_value_is_named_and_never_zero | 绿=missing_value_is_named_and_never_zero
//! // TDD-PROBE: DataPointValue::value | 变异：缺失时返回 Some(0.0) | 红=missing_value_is_named_and_never_zero | 绿=missing_value_is_named_and_never_zero
//! // TDD-PROBE: DataPointValue::is_missing | 变异：恒返回 false | 红=missing_value_is_named_and_never_zero | 绿=missing_value_is_named_and_never_zero
//! // TDD-PROBE: DataPointValue::missing_reason | 变异：有值时也返回 Some | 红=missing_value_is_named_and_never_zero | 绿=missing_value_is_named_and_never_zero
//! // TDD-PROBE: JapanCbDataPoint::new | 变异：跳过 validate_data_point | 红=data_point_validates_granularity | 绿=data_point_validates_granularity
//! // TDD-PROBE: JapanCbDataPoint::data_point_id | 变异：丢掉修订标识段 | 红=data_point_validates_granularity | 绿=data_point_validates_granularity
//! // TDD-PROBE: validate_data_point | 变异：不比对频率与期间粒度 | 红=data_point_validates_granularity | 绿=data_point_validates_granularity
//! // TDD-PROBE: ensure_no_converted_or_derived_field | 变异：返回 Ok | 红=converted_fields_are_out_of_scope | 绿=converted_fields_are_out_of_scope
//! // TDD-PROBE: JapanCbAuthorization::is_authorized | 变异：Denied 也返回 true | 红=authorization_is_fail_closed | 绿=authorization_is_fail_closed
//! // TDD-PROBE: JapanCbAuthorization::denial_reason | 变异：理由返回空串 | 红=authorization_is_fail_closed | 绿=authorization_is_fail_closed
//! // TDD-PROBE: JapanCbAuthorizationEvidence::new | 变异：丢弃 signer | 红=authorization_is_fail_closed | 绿=authorization_is_fail_closed
//! // TDD-PROBE: decide_authorization | 变异：证据缺失时放行 | 红=authorization_is_fail_closed | 绿=authorization_is_fail_closed
//! // TDD-PROBE: registered_evidence | 变异：返回合成证据 | 红=registered_evidence_is_absent | 绿=registered_evidence_is_absent
//! // TDD-PROBE: current_authorization | 变异：恒返回 Authorized | 红=registered_evidence_is_absent | 绿=registered_evidence_is_absent
//! // TDD-PROBE: ensure_authorized | 变异：Denied 返回 Ok | 红=ensure_authorized_maps_denial | 绿=ensure_authorized_maps_denial
//! // TDD-PROBE: publication_semantics | 变异：返回 Formal | 红=publication_triple_is_fixed | 绿=publication_triple_is_fixed
//! // TDD-PROBE: publication_for_period | 变异：改写传入的期间 | 红=publication_triple_is_fixed | 绿=publication_triple_is_fixed
//! // TDD-PROBE: parse_japanx_data_points | 变异：忽略未知字段 | 红=parser_rejects_unknown_fields | 绿=parser_rejects_unknown_fields
//! // TDD-PROBE: parse_japanx_source_definition | 变异：接受 rate_limit_rps | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: JapanCbSourceDefinition::kind | 变异：返回固定字符串 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: JapanCbSourceDefinition::encoding | 变异：返回 utf-8 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: JapanCbSourceDefinition::unit | 变异：返回空单位 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: JapanCbSourceDefinition::scale | 变异：scale 恒返回 0 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: JapanCbSourceDefinition::metric_codes | 变异：返回空切片 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: SOURCE_KIND_CSV_DOWNLOAD | 变异：写成 csv 下载 | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: SOURCE_ENCODING_SHIFT_JIS | 变异：写成 shift-jis | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope
//! // TDD-PROBE: PARSER_INPUT_ENCODING | 变异：写成 shift_jis | 红=source_definition_parser_is_declared_scope | 绿=source_definition_parser_is_declared_scope

use japanx::{
    claim_curve_authority, claim_forwarded_series_sovereignty, current_authorization,
    decide_authorization, ensure_authorized, ensure_no_converted_or_derived_field,
    is_declared_metric_code, parse_japanx_data_points, parse_japanx_source_definition,
    publication_for_period, publication_semantics, registered_evidence, reject_excluded_fx_symbol,
    reject_silent_fx_substitution, validate_data_point, validate_metric_code, AvailabilityEvidence,
    DataPointValue, Date, Frequency, JapanCbAuthorization, JapanCbAuthorizationEvidence,
    JapanCbDataPoint, JapanCbError, JapanCbErrorKind, JapanCbMissingReason, MetricCategory, Period,
    PitEligibility, Revision, TimePrecision, Unit, DECLARED_METRIC_CODES, EXCLUDED_FX_SYMBOLS,
    FRED_EXCLUSIVE_FX_SERIES, JP_BS_ETF_HOLDINGS, JP_BS_JGB_HOLDINGS, JP_BS_TOTAL_ASSETS,
    JP_CA_EXCESS_RESERVES, JP_CA_TOTAL, JP_FX_INTERVENTION, JP_IR_POLICY_RATE, JP_IR_TONA,
    JP_MB_TOTAL, JP_MO_JGB_PURCHASE, JP_MS_M2, JP_TK_LARGE_MFG_DI, PARSER_INPUT_ENCODING,
    SOURCE_ENCODING_SHIFT_JIS, SOURCE_KIND_CSV_DOWNLOAD, SOURCE_UNIT_HUNDRED_MILLION_YEN,
};

fn unit() -> Unit {
    Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN).expect("合法单位")
}

fn day() -> Date {
    Date::new(2026, 9, 18).expect("合法日期")
}

fn every_error() -> Vec<JapanCbError> {
    vec![
        JapanCbError::Invalid("x".to_owned()),
        JapanCbError::Missing("x".to_owned()),
        JapanCbError::AuthorizationDenied("x".to_owned()),
        JapanCbError::RoutedElsewhere("x".to_owned()),
        JapanCbError::WriteAuthorityDenied("x".to_owned()),
        JapanCbError::SemanticallyRejected("x".to_owned()),
        JapanCbError::NotApplicable("x".to_owned()),
        JapanCbError::Invariant("x".to_owned()),
    ]
}

fn data_point(period: Period, frequency: Frequency, value: DataPointValue) -> JapanCbDataPoint {
    JapanCbDataPoint::new(
        validate_metric_code(JP_BS_TOTAL_ASSETS).expect("合法 metric_code"),
        period,
        frequency,
        value,
        unit(),
        None,
    )
    .expect("合法数据点")
}

#[test]
fn error_kind_maps_every_variant_distinctly() {
    let kinds: Vec<JapanCbErrorKind> = every_error().iter().map(JapanCbError::kind).collect();
    assert_eq!(
        kinds,
        [
            JapanCbErrorKind::Invalid,
            JapanCbErrorKind::Missing,
            JapanCbErrorKind::AuthorizationDenied,
            JapanCbErrorKind::RoutedElsewhere,
            JapanCbErrorKind::WriteAuthorityDenied,
            JapanCbErrorKind::SemanticallyRejected,
            JapanCbErrorKind::NotApplicable,
            JapanCbErrorKind::Invariant,
        ]
    );
}

#[test]
fn only_invariant_is_retryable() {
    for error in every_error() {
        assert_eq!(
            error.is_retryable(),
            error.kind() == JapanCbErrorKind::Invariant,
            "{error:?}"
        );
    }
}

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
fn metric_code_shape_is_validated() {
    for code in DECLARED_METRIC_CODES {
        assert_eq!(validate_metric_code(code).expect(code).as_str(), code);
    }
    for bad in [
        "",
        "JP.BS",
        "US.BS.TOTAL_ASSETS",
        "JP.XX.Y",
        "JP.BS.",
        "JP.BS.A B",
        "JP.BS.A.B.C",
    ] {
        assert!(validate_metric_code(bad).is_err(), "{bad:?}");
    }
    // 子维度形态合法。
    assert!(validate_metric_code("JP.BS.TOTAL_ASSETS.SUB").is_ok());
}

#[test]
fn declared_membership_is_checked_separately() {
    let declared = validate_metric_code(JP_MS_M2).expect("合法");
    assert_eq!(declared.as_str(), JP_MS_M2);
    assert!(declared.is_declared());
    assert!(is_declared_metric_code(JP_MS_M2));
    assert_eq!(declared.to_string(), JP_MS_M2);

    let well_formed_but_undeclared =
        validate_metric_code("JP.TK.LARGE_NON_MFG_DI").expect("规范形合法");
    assert!(!well_formed_but_undeclared.is_declared());
    assert!(!is_declared_metric_code("JP.TK.LARGE_NON_MFG_DI"));
}

#[test]
fn eight_categories_are_listed() {
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
    for code in DECLARED_METRIC_CODES {
        assert_eq!(
            validate_metric_code(code)
                .expect(code)
                .category()
                .expect("类别可解析")
                .code()
                .len(),
            2,
            "{code}"
        );
    }
}

#[test]
fn excluded_fx_symbols_are_refused() {
    assert_eq!(EXCLUDED_FX_SYMBOLS, ["JPY=X", "USDJPY=X"]);
    assert_eq!(FRED_EXCLUSIVE_FX_SERIES, "DEXJPUS");
    for symbol in EXCLUDED_FX_SYMBOLS {
        assert_eq!(
            reject_excluded_fx_symbol(symbol)
                .expect_err("本库不得主张该主源")
                .kind(),
            JapanCbErrorKind::WriteAuthorityDenied
        );
    }
    assert!(reject_excluded_fx_symbol(FRED_EXCLUSIVE_FX_SERIES).is_ok());
}

#[test]
fn silent_fx_substitution_is_rejected() {
    for pair in [
        ("JPY=X", FRED_EXCLUSIVE_FX_SERIES),
        ("USDJPY=X", FRED_EXCLUSIVE_FX_SERIES),
    ] {
        assert_eq!(
            reject_silent_fx_substitution(pair.0, pair.1)
                .expect_err("近义非同 ID 不得互换")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }
    assert!(reject_silent_fx_substitution(JP_MB_TOTAL, JP_MS_M2).is_ok());
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

#[test]
fn period_constructors_validate_ranges() {
    assert_eq!(Period::day(day()).key(), "2026-09-18");
    assert!(Period::month(2026, 0).is_err());
    assert!(Period::month(2026, 13).is_err());
    assert!(Period::quarter(2026, 0).is_err());
    assert!(Period::quarter(2026, 5).is_err());
    assert!(Period::year(0).is_err());
    assert!(Period::ten_day(2026, 9, 0).is_err());
    assert!(Period::ten_day(2026, 9, 4).is_err());
    assert!(Period::ten_day(2026, 9, 3).is_ok());
    assert!(Period::event(day()).key().starts_with("event:"));
    assert_eq!(Period::event(day()).key(), "event:2026-09-18");
}

#[test]
fn period_keys_round_trip() {
    let cases = [
        Period::day(day()),
        Period::ten_day(2026, 9, 2).expect("合法旬期间"),
        Period::month(2026, 9).expect("合法月期间"),
        Period::quarter(2026, 3).expect("合法季期间"),
        Period::year(2026).expect("合法年期间"),
        Period::event(day()),
    ];
    for period in cases {
        let key = period.key();
        assert_eq!(Period::parse(&key).expect("可回读"), period, "key={key}");
    }
    assert_eq!(
        Period::ten_day(2026, 9, 2).expect("合法").key(),
        "2026-09-T2"
    );
    for bad in ["2026-2-3", "2026/09/18", "2026-09-T4", "2026-Q5", ""] {
        assert!(Period::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn period_frequency_covers_declared_granularities() {
    assert_eq!(Period::day(day()).frequency(), Frequency::Daily);
    assert_eq!(
        Period::ten_day(2026, 9, 1).expect("合法").frequency(),
        Frequency::TenDay
    );
    assert_eq!(
        Period::month(2026, 9).expect("合法").frequency(),
        Frequency::Monthly
    );
    assert_eq!(
        Period::quarter(2026, 3).expect("合法").frequency(),
        Frequency::Quarterly
    );
    assert_eq!(
        Period::year(2026).expect("合法").frequency(),
        Frequency::Annual
    );
    assert_eq!(Period::event(day()).frequency(), Frequency::Event);
}

#[test]
fn frequency_names_round_trip() {
    assert_eq!(Frequency::ALL.len(), 8);
    let names: Vec<&str> = Frequency::ALL.iter().map(Frequency::name).collect();
    assert_eq!(
        names,
        [
            "Daily",
            "TenDay",
            "Weekly",
            "Monthly",
            "Quarterly",
            "Annual",
            "Event",
            "Irregular"
        ]
    );
    for frequency in Frequency::ALL {
        assert_eq!(
            Frequency::from_name(frequency.name()).expect("可回读"),
            frequency
        );
    }
    assert!(Frequency::from_name("daily").is_err());
}

#[test]
fn source_tokens_are_limited_to_the_declared_five() {
    let declared: Vec<&str> = Frequency::SOURCE_DECLARED
        .iter()
        .map(Frequency::name)
        .collect();
    assert_eq!(
        declared,
        ["Daily", "TenDay", "Monthly", "Quarterly", "Annual"]
    );
    for frequency in Frequency::SOURCE_DECLARED {
        assert_eq!(
            Frequency::from_source_token(frequency.name()).expect("清单声明值"),
            frequency
        );
    }
    for undeclared in ["Weekly", "Event", "Irregular"] {
        assert_eq!(
            Frequency::from_source_token(undeclared)
                .expect_err(undeclared)
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }
}

#[test]
fn date_strict_iso_and_leap_years() {
    let date = Date::parse("2026-09-08").expect("合法日期");
    assert_eq!((date.year(), date.month(), date.day()), (2026, 9, 8));
    assert_eq!(date.to_iso(), "2026-09-08");
    assert_eq!(Date::parse(&date.to_iso()).expect("可回读"), date);

    for bad in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-02-03T00:00:00Z",
    ] {
        assert!(Date::parse(bad).is_err(), "{bad}");
    }
    assert!(Date::is_leap_year(2000));
    assert!(!Date::is_leap_year(1900));
    assert_eq!(Date::days_in_month(2024, 2).expect("2 月"), 29);
    assert!(Date::days_in_month(2026, 13).is_err());
    assert!(Date::new(2026, 2, 29).is_err());
}

#[test]
fn unit_and_revision_keep_source_text() {
    assert_eq!(SOURCE_UNIT_HUNDRED_MILLION_YEN, "100 million yen");
    let unit = Unit::try_new("  100 million yen  ").expect("合法单位");
    assert_eq!(unit.text(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
    assert_eq!(unit.to_string(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
    assert!(Unit::try_new("   ").is_err());

    let revision = Revision::try_new(" R2 ").expect("合法修订标识");
    assert_eq!(revision.text(), "R2");
    assert_eq!(revision.to_string(), "R2");
    assert!(Revision::try_new("").is_err());
}

#[test]
fn missing_value_is_named_and_never_zero() {
    let absent = data_point(
        Period::day(day()),
        Frequency::Daily,
        DataPointValue::Missing(JapanCbMissingReason::SourceStatus(
            "SYNTH_STATUS".to_owned(),
        )),
    );
    assert_eq!(absent.value.value(), None, "缺失不得转 0");
    assert!(absent.value.is_missing());
    assert!(absent.value.missing_reason().is_some());

    let present = data_point(
        Period::day(day()),
        Frequency::Daily,
        DataPointValue::Value(0.5),
    );
    assert_eq!(present.value.value(), Some(0.5));
    assert!(!present.value.is_missing());
    assert_eq!(present.value.missing_reason(), None);
    assert!(!JapanCbMissingReason::Unspecified.describe().is_empty());
}

#[test]
fn data_point_validates_granularity() {
    let plain = data_point(
        Period::month(2026, 8).expect("合法期间"),
        Frequency::Monthly,
        DataPointValue::Value(1.0),
    );
    assert_eq!(plain.data_point_id(), "JP.BS.TOTAL_ASSETS:2026-08:none");
    assert!(validate_data_point(&plain).is_ok());

    let revised = JapanCbDataPoint::new(
        validate_metric_code(JP_BS_TOTAL_ASSETS).expect("合法"),
        Period::month(2026, 8).expect("合法期间"),
        Frequency::Monthly,
        DataPointValue::Value(1.0),
        unit(),
        Some(Revision::try_new("R2").expect("合法修订标识")),
    )
    .expect("合法数据点");
    assert_ne!(plain.data_point_id(), revised.data_point_id());

    let mismatched = JapanCbDataPoint::new(
        validate_metric_code(JP_BS_TOTAL_ASSETS).expect("合法"),
        Period::year(2026).expect("合法期间"),
        Frequency::Daily,
        DataPointValue::Value(1.0),
        unit(),
        None,
    );
    assert_eq!(
        mismatched.expect_err("粒度不一致必须拒绝").kind(),
        JapanCbErrorKind::SemanticallyRejected
    );
}

#[test]
fn converted_fields_are_out_of_scope() {
    assert_eq!(
        ensure_no_converted_or_derived_field()
            .expect_err("换算 / 派生 / 存储字段不属本层")
            .kind(),
        JapanCbErrorKind::NotApplicable
    );
}

#[test]
fn authorization_is_fail_closed() {
    // 缺失 / 范围不明 / 签署者不明 / 有效期不明 / 已过期 —— 一律拒绝。
    assert!(!decide_authorization(None, day()).is_authorized());

    let shapes = [
        JapanCbAuthorizationEvidence::new(
            "ref",
            Some("signer"),
            None,
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        ),
        JapanCbAuthorizationEvidence::new(
            "ref",
            None,
            Some("scope"),
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        ),
        JapanCbAuthorizationEvidence::new("ref", Some("signer"), Some("scope"), None),
        JapanCbAuthorizationEvidence::new(
            "ref",
            Some("signer"),
            Some("scope"),
            Some(Date::new(2026, 1, 1).expect("合法日期")),
        ),
        JapanCbAuthorizationEvidence::new(
            "  ",
            Some("signer"),
            Some("scope"),
            Some(Date::new(2030, 1, 1).expect("合法日期")),
        ),
    ];
    for evidence in shapes {
        let verdict = decide_authorization(Some(&evidence), day());
        assert!(!verdict.is_authorized(), "{evidence:?}");
        assert!(verdict
            .denial_reason()
            .is_some_and(|reason| !reason.trim().is_empty()));
    }

    // 合成证据只用于验证判定形状，不构成任何真实授权证据。
    let complete = JapanCbAuthorizationEvidence::new(
        "synthetic-evidence-ref",
        Some("synthetic-signer"),
        Some("offline_parse_and_types"),
        Some(Date::new(2030, 1, 1).expect("合法日期")),
    );
    match decide_authorization(Some(&complete), day()) {
        JapanCbAuthorization::Authorized { scope } => assert_eq!(scope, "offline_parse_and_types"),
        other => panic!("完整证据应放行：{other:?}"),
    }
}

#[test]
fn registered_evidence_is_absent() {
    assert!(registered_evidence().is_none(), "本源不得凭空构造证据");
    assert!(!current_authorization(day()).is_authorized());
}

#[test]
fn ensure_authorized_maps_denial() {
    let error = ensure_authorized(&current_authorization(day())).expect_err("unknown 授权必须拒绝");
    assert_eq!(error.kind(), JapanCbErrorKind::AuthorizationDenied);
    assert!(!error.is_retryable());
    let allowed = JapanCbAuthorization::Authorized {
        scope: "synthetic-scope".to_owned(),
    };
    assert!(ensure_authorized(&allowed).is_ok());
}

#[test]
fn publication_triple_is_fixed() {
    assert_eq!(
        publication_semantics(),
        (
            TimePrecision::Date,
            AvailabilityEvidence::Inferred,
            PitEligibility::NotEligible
        )
    );
    let (echoed, precision, evidence, eligibility) = publication_for_period(day());
    assert_eq!(echoed, day());
    assert_eq!(precision, TimePrecision::Date);
    assert_eq!(evidence, AvailabilityEvidence::Inferred);
    assert_eq!(eligibility, PitEligibility::NotEligible);
}

#[test]
fn parser_rejects_unknown_fields() {
    let good = include_str!("fixtures/jp_data_points.json");
    assert_eq!(
        parse_japanx_data_points(good)
            .expect("合成夹具必须可解析")
            .len(),
        4
    );

    let with_unknown = good.replace(
        "\"revision\": null",
        "\"revision\": null, \"endpoint\": \"nope\"",
    );
    assert_eq!(
        parse_japanx_data_points(&with_unknown)
            .expect_err("未知字段必须原子失败")
            .kind(),
        JapanCbErrorKind::Invalid
    );

    // 缺失必需字段同样原子失败。
    let missing_unit = good.replace(
        "\"unit\": \"100 million yen\",\n      \"revision\": null",
        "\"revision\": null",
    );
    assert!(parse_japanx_data_points(&missing_unit).is_err());
}

#[test]
fn source_definition_parser_is_declared_scope() {
    assert_eq!(SOURCE_KIND_CSV_DOWNLOAD, "csv_download");
    assert_eq!(SOURCE_ENCODING_SHIFT_JIS, "shift_jis");
    assert_eq!(PARSER_INPUT_ENCODING, "utf-8");

    let fixture = include_str!("fixtures/jp_source_definition.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("夹具是 JSON");
    let toml_text = parsed["toml"].as_str().expect("toml 字段是字符串");
    let definition = parse_japanx_source_definition(toml_text).expect("合成源定义应可解析");
    assert_eq!(definition.kind(), SOURCE_KIND_CSV_DOWNLOAD);
    assert_eq!(definition.encoding(), SOURCE_ENCODING_SHIFT_JIS);
    assert_eq!(definition.unit().text(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
    assert_eq!(definition.scale(), 1.0);
    assert_eq!(definition.metric_codes().len(), 3);

    // 限流数字必须被原子拒绝（授权前不写具体限流数字）。
    let with_rate_limit = format!("rate_limit_rps = 5\n{toml_text}");
    assert_eq!(
        parse_japanx_source_definition(&with_rate_limit)
            .expect_err("限流数字必须被原子拒绝")
            .kind(),
        JapanCbErrorKind::Invalid
    );

    // 未声明的 metric_code 与错误源形态必须被拒绝。
    let undeclared = toml_text.replace("JP.CA.EXCESS_RESERVES", "JP.TK.LARGE_NON_MFG_DI");
    assert!(parse_japanx_source_definition(&undeclared).is_err());
    let wrong_kind = toml_text.replace("csv_download", "json_api");
    assert!(parse_japanx_source_definition(&wrong_kind).is_err());
}
