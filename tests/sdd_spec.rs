#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! SDD 规格对照（特性 005）：把 `docs/标准.md` 的每个 `##` 章节转成可执行断言。
//!
//! 章节与断言函数须与 `docs/标准.md` 的 `##` 章节 1:1（检查器按标题逐字比对）。
//!
//! // SPEC-MAP: S-1 | 1. metric_code 标准 | assert_metric_code_standard
//! // SPEC-MAP: S-2 | 2. 类别与期间标准 | assert_category_and_period_standard
//! // SPEC-MAP: S-3 | 3. 单位与值标准 | assert_unit_and_value_standard
//! // SPEC-MAP: S-4 | 4. 曲线与主源边界标准 | assert_curve_and_primary_source_boundary
//! // SPEC-MAP: S-5 | 5. 授权与 publication 标准 | assert_authorization_and_publication_standard
//! // SPEC-MAP: S-6 | 6. 输入形态与解析边界 | assert_input_form_and_parse_boundary
//! // SPEC-MAP: S-7 | 7. 合成夹具声明 | assert_synthetic_fixture_declaration
//! // SPEC-MAP: S-8 | 8. 非目标与门禁 | assert_non_goals_and_gates

use std::path::{Path, PathBuf};

use japanx::{
    claim_curve_authority, claim_forwarded_series_sovereignty, current_authorization,
    ensure_no_converted_or_derived_field, is_declared_metric_code, parse_japanx_data_points,
    parse_japanx_source_definition, publication_semantics, reject_excluded_fx_symbol,
    reject_silent_fx_substitution, validate_data_point, validate_metric_code, AvailabilityEvidence,
    DataPointValue, Date, Frequency, JapanCbDataPoint, JapanCbErrorKind, JapanCbMissingReason,
    MetricCategory, Period, PitEligibility, TimePrecision, Unit, DECLARED_METRIC_CODES,
    FRED_EXCLUSIVE_FX_SERIES, PARSER_INPUT_ENCODING, SOURCE_ENCODING_SHIFT_JIS,
    SOURCE_UNIT_HUNDRED_MILLION_YEN,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("目录可读") {
        let path = entry.expect("目录项可读").path();
        if path.is_dir() {
            collect_rust_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn unit() -> Unit {
    Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN).expect("合法单位")
}

fn day() -> Date {
    Date::new(2026, 9, 18).expect("合法日期")
}

fn data_point(period: Period, frequency: Frequency, value: DataPointValue) -> JapanCbDataPoint {
    JapanCbDataPoint::new(
        validate_metric_code(DECLARED_METRIC_CODES[0]).expect("合法 metric_code"),
        period,
        frequency,
        value,
        unit(),
        None,
    )
    .expect("合法数据点")
}

/// S-1：声明集只有清单 §1.1 的 12 个 metric_code；规范形校验与声明判定分离。
#[test]
fn assert_metric_code_standard() {
    assert_eq!(DECLARED_METRIC_CODES.len(), 12);
    for code in DECLARED_METRIC_CODES {
        let parsed = validate_metric_code(code).expect(code);
        assert_eq!(parsed.as_str(), code);
        assert!(parsed.is_declared());
        assert!(is_declared_metric_code(code));
    }
    assert!(validate_metric_code("JP.BS").is_err(), "段数不足");
    assert_eq!(
        validate_metric_code("JP.XX.Y")
            .expect_err("类别未声明")
            .kind(),
        JapanCbErrorKind::SemanticallyRejected
    );
    // 规范形合法但未声明：结构通过、声明为假。
    assert!(!validate_metric_code("JP.TK.LARGE_NON_MFG_DI")
        .expect("规范形合法")
        .is_declared());
}

/// S-2：8 个类别码；期间与频率覆盖清单声明的 5 个取值（含旬）。
#[test]
fn assert_category_and_period_standard() {
    assert_eq!(
        MetricCategory::ALL
            .iter()
            .map(MetricCategory::code)
            .collect::<Vec<&str>>(),
        ["BS", "IR", "MB", "MS", "CA", "MO", "FX", "TK"]
    );
    assert!(MetricCategory::from_code("ZZ").is_err());

    let declared: Vec<&str> = Frequency::SOURCE_DECLARED
        .iter()
        .map(Frequency::name)
        .collect();
    assert_eq!(
        declared,
        ["Daily", "TenDay", "Monthly", "Quarterly", "Annual"]
    );
    assert!(Frequency::from_source_token("Weekly").is_err());

    for period in [
        Period::day(day()),
        Period::ten_day(2026, 9, 2).expect("合法旬期间"),
        Period::month(2026, 9).expect("合法月期间"),
        Period::quarter(2026, 3).expect("合法季期间"),
        Period::year(2026).expect("合法年期间"),
        Period::event(day()),
    ] {
        assert_eq!(Period::parse(&period.key()).expect("可回读"), period);
    }
    assert_eq!(
        Period::ten_day(2026, 9, 2).expect("合法").key(),
        "2026-09-T2"
    );
    for bad in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-09-T4",
        "2026-Q5",
    ] {
        assert!(Period::parse(bad).is_err(), "{bad}");
    }
}

/// S-3：源单位原样保留；缺失具名；换算 / 派生 / 存储字段不属本层。
#[test]
fn assert_unit_and_value_standard() {
    assert_eq!(SOURCE_UNIT_HUNDRED_MILLION_YEN, "100 million yen");
    assert_eq!(unit().text(), SOURCE_UNIT_HUNDRED_MILLION_YEN);

    let absent = data_point(
        Period::month(2026, 8).expect("合法期间"),
        Frequency::Monthly,
        DataPointValue::Missing(JapanCbMissingReason::Unspecified),
    );
    assert_eq!(absent.value.value(), None, "缺失不得转 0");
    assert!(absent.value.missing_reason().is_some());

    let mismatched = JapanCbDataPoint::new(
        validate_metric_code(DECLARED_METRIC_CODES[1]).expect("合法"),
        Period::year(2026).expect("合法期间"),
        Frequency::Daily,
        DataPointValue::Value(1.0),
        unit(),
        None,
    );
    assert!(mismatched.is_err(), "频率与期间粒度必须自洽");
    assert!(validate_data_point(&data_point(
        Period::month(2026, 8).expect("合法期间"),
        Frequency::Monthly,
        DataPointValue::Value(1.0),
    ))
    .is_ok());

    assert_eq!(
        ensure_no_converted_or_derived_field()
            .expect_err("换算 / 派生 / 存储字段不属本层")
            .kind(),
        JapanCbErrorKind::NotApplicable
    );
}

/// S-4：曲线归 `yieldx`；`JPY=X` / `USDJPY=X` 归 `fredx` 独占；主权待裁。
#[test]
fn assert_curve_and_primary_source_boundary() {
    assert_eq!(
        claim_curve_authority().expect_err("曲线归 yieldx").kind(),
        JapanCbErrorKind::RoutedElsewhere
    );
    for symbol in ["JPY=X", "USDJPY=X"] {
        assert_eq!(
            reject_excluded_fx_symbol(symbol)
                .expect_err("不得主张主源")
                .kind(),
            JapanCbErrorKind::WriteAuthorityDenied
        );
        assert_eq!(
            reject_silent_fx_substitution(symbol, FRED_EXCLUSIVE_FX_SERIES)
                .expect_err("不得互换")
                .kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }
    assert_eq!(
        claim_forwarded_series_sovereignty()
            .expect_err("主权待裁")
            .kind(),
        JapanCbErrorKind::WriteAuthorityDenied
    );
}

/// S-5：授权 fail-closed；publication 恒 `Date` + `Inferred` + `NotEligible`。
#[test]
fn assert_authorization_and_publication_standard() {
    assert!(!current_authorization(day()).is_authorized());
    assert_eq!(
        publication_semantics(),
        (
            TimePrecision::Date,
            AvailabilityEvidence::Inferred,
            PitEligibility::NotEligible
        )
    );
}

/// S-6：两个离线入口；未知字段原子失败、重复身份拒绝、限流数字原子失败、编码契约。
#[test]
fn assert_input_form_and_parse_boundary() {
    assert_eq!(PARSER_INPUT_ENCODING, "utf-8");
    assert_eq!(SOURCE_ENCODING_SHIFT_JIS, "shift_jis");

    let document = include_str!("fixtures/jp_data_points.json");
    let points = parse_japanx_data_points(document).expect("合成夹具必须可解析");
    assert_eq!(points.len(), 4);
    assert!(points[0].metric_code.is_declared());
    assert_eq!(points[0].unit.text(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
    assert!(points[2].value.is_missing());

    assert!(parse_japanx_data_points(r#"{ "data_points": [] }"#).is_err());

    let duplicated = r#"{ "_synthetic": true, "_note": "合成样本",
        "data_points": [
          { "metric_code": "JP.MS.M2", "period": "2026-08", "frequency": "Monthly",
            "value": 1.0, "unit": "100 million yen" },
          { "metric_code": "JP.MS.M2", "period": "2026-08", "frequency": "Monthly",
            "value": 2.0, "unit": "100 million yen" }
        ] }"#;
    assert_eq!(
        parse_japanx_data_points(duplicated)
            .expect_err("重复身份必须拒绝")
            .kind(),
        JapanCbErrorKind::SemanticallyRejected
    );

    let fixture = include_str!("fixtures/jp_source_definition.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("夹具是 JSON");
    let toml_text = parsed["toml"].as_str().expect("toml 字段是字符串");
    assert!(parse_japanx_source_definition(toml_text).is_ok());
    assert_eq!(
        parse_japanx_source_definition(&format!("rate_limit_rps = 5\n{toml_text}"))
            .expect_err("限流数字必须被原子拒绝")
            .kind(),
        JapanCbErrorKind::Invalid
    );
}

/// S-7：全部夹具为合成样本，不是真实源数据，不构成证据。
#[test]
fn assert_synthetic_fixture_declaration() {
    let fixtures = manifest_dir().join("tests").join("fixtures");
    let mut json_files = Vec::new();
    for entry in std::fs::read_dir(&fixtures).expect("夹具目录可读") {
        let path = entry.expect("目录项可读").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            json_files.push(path);
        }
    }
    assert!(!json_files.is_empty(), "夹具目录不得为空");
    for path in json_files {
        let text = std::fs::read_to_string(&path).expect("夹具可读");
        assert!(text.contains("\"_synthetic\": true"), "{path:?}");
        assert!(text.contains("\"_note\""), "{path:?}");
        for forbidden in ["实测", "核验 PASS", "证据等级"] {
            assert!(!text.contains(forbidden), "{path:?} 含禁用表述 {forbidden}");
        }
    }
}

/// S-8：非目标（零端点 / 零 HTTP 依赖 / 零限流数字 / 零凭据 / 零跨仓依赖）与门禁面。
#[test]
fn assert_non_goals_and_gates() {
    let mut sources = Vec::new();
    collect_rust_sources(&manifest_dir().join("src"), &mut sources);
    assert!(!sources.is_empty(), "src 不得为空");
    for path in &sources {
        let text = std::fs::read_to_string(path).expect("源文件可读");
        for forbidden in ["http://", "https://", "env::var", "from_env"] {
            assert!(!text.contains(forbidden), "{path:?} 含禁用片段 {forbidden}");
        }
        // 授权前不写具体限流数字：**生产代码**里不得出现限流字段。
        // 判定只看非注释行、且排除 `#[cfg(test)]` 段（测试要制造该负向输入）。
        let production = text.split("#[cfg(test)]").next().unwrap_or(&text);
        for (index, line) in production.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            assert!(
                !line.contains("rate_limit"),
                "{path:?} 第 {} 行出现限流字段",
                index + 1
            );
        }
    }

    // 零 HTTP 客户端 / 零跨仓依赖：`[dependencies]` 段只允许已登记的最小依赖集。
    let manifest =
        std::fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("Cargo.toml 可读");
    assert!(!manifest.contains("path = \"../"), "零跨仓依赖");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split('[').next())
        .expect("存在 [dependencies] 段");
    let allowed = ["serde", "serde_json", "thiserror", "csv", "toml"];
    for line in dependencies.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line
            .split(|c: char| c == '=' || c.is_whitespace())
            .next()
            .expect("依赖名");
        assert!(allowed.contains(&name), "未登记的依赖：{name}");
    }

    assert!(manifest.contains("[[bench]]"));
    assert!(manifest.contains("harness = false"));
    for name in ["tdd_contracts.rs", "sdd_spec.rs", "aidd_boundary.rs"] {
        assert!(
            manifest_dir().join("tests").join(name).is_file(),
            "缺少 tests/{name}"
        );
    }
}
