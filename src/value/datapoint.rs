//! 观测值对象：单位、修订标识、缺失表达与主观测类型。
//!
//! **源侧单位原样保留**：清单 §1.2/§3 的源单位是 `100 million yen`（日元亿）；
//! 单位换算与派生指标一律归下游，本层不做。

use std::fmt;

use crate::error::{JapanCbError, JapanCbResult};
use crate::value::metric::JapanCbMetricCode;
use crate::value::period::{Frequency, Period};

/// 清单声明的源侧单位（日元亿）。
pub const SOURCE_UNIT_HUNDRED_MILLION_YEN: &str = "100 million yen";

/// 源侧单位。
///
/// 保留**源单位原文**，本层不做任何换算，也不内置单位枚举：清单只给出一个单位，
/// 凭空虚造集合就是编造。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit(String);

impl Unit {
    /// 构造并校验（去首尾空白后非空）。
    pub fn try_new(text: &str) -> JapanCbResult<Self> {
        if text.trim().is_empty() {
            return Err(JapanCbError::Missing("源侧单位为空".to_owned()));
        }
        Ok(Self(text.trim().to_owned()))
    }

    /// 源单位原文。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// 源侧修订标识原文。
///
/// 清单 §4 有 `revision` 字段但未定义其形态（序号 / 日期 / 标记皆有可能），
/// 故本库只**原样承载**，不解释、不派生；无官方 vintage 面时整字段为 `None`。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Revision(String);

impl Revision {
    /// 构造并校验（去首尾空白后非空）。
    pub fn try_new(text: &str) -> JapanCbResult<Self> {
        if text.trim().is_empty() {
            return Err(JapanCbError::Missing("修订标识为空".to_owned()));
        }
        Ok(Self(text.trim().to_owned()))
    }

    /// 修订标识原文。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// 缺失原因。
///
/// 这是**本库自有分类**，不是源侧 code list 的实现：本库不知道也不声称掌握源侧
/// 状态码字典，故只把源侧状态原文原样带出。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JapanCbMissingReason {
    /// 源侧给出了状态标记（原文保留）。
    SourceStatus(String),
    /// 源侧没有给出可用的状态标记。
    Unspecified,
}

impl JapanCbMissingReason {
    /// 可读描述。
    #[must_use]
    pub fn describe(&self) -> &str {
        match self {
            Self::SourceStatus(status) => status,
            Self::Unspecified => "源侧未给出状态标记",
        }
    }
}

/// 观测值：有值或具名缺失。
///
/// 缺失**绝不**静默转 0。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum DataPointValue {
    /// 源给出的数值。
    Value(f64),
    /// 缺失，附具名原因。
    Missing(JapanCbMissingReason),
}

impl DataPointValue {
    /// 有值时返回数值。
    #[must_use]
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Value(value) => Some(*value),
            Self::Missing(_) => None,
        }
    }

    /// 是否缺失。
    #[must_use]
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// 缺失原因；有值时为 `None`。
    #[must_use]
    pub fn missing_reason(&self) -> Option<&JapanCbMissingReason> {
        match self {
            Self::Value(_) => None,
            Self::Missing(reason) => Some(reason),
        }
    }
}

/// 一条日本央行源事实数据点。
///
/// 字段面取自清单 §4 的 `DataPoint`，但**有意排除**三个字段（见
/// [`ensure_no_converted_or_derived_field`]）：`value_usd`（单位换算）、
/// `fingerprint`（内容哈希）、`raw_object_key`（存储层寻址）。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct JapanCbDataPoint {
    /// 清单 §1.1 的 `JP.*` metric_code。
    pub metric_code: JapanCbMetricCode,
    /// 业务期间（清单 §4 的 `obs_time` 落点）。
    pub period: Period,
    /// 频率（清单 §4 的 `Period{Y}` 落点）。
    pub frequency: Frequency,
    /// 观测值（有值或具名缺失）。
    pub value: DataPointValue,
    /// 源侧单位（原样保留）。
    pub unit: Unit,
    /// 修订标识。无官方 vintage 面时**必须**为 `None`，不得伪造。
    pub revision: Option<Revision>,
}

impl JapanCbDataPoint {
    /// 构造并校验一条数据点。
    pub fn new(
        metric_code: JapanCbMetricCode,
        period: Period,
        frequency: Frequency,
        value: DataPointValue,
        unit: Unit,
        revision: Option<Revision>,
    ) -> JapanCbResult<Self> {
        let data_point = Self {
            metric_code,
            period,
            frequency,
            value,
            unit,
            revision,
        };
        validate_data_point(&data_point)?;
        Ok(data_point)
    }

    /// 数据点身份串：`metric_code:期间:修订标识`。
    #[must_use]
    pub fn data_point_id(&self) -> String {
        let revision = self
            .revision
            .as_ref()
            .map_or_else(|| "none".to_owned(), |revision| revision.text().to_owned());
        format!(
            "{}:{}:{}",
            self.metric_code.as_str(),
            self.period.key(),
            revision
        )
    }
}

/// 校验数据点的完整性与内部一致性。
pub fn validate_data_point(data_point: &JapanCbDataPoint) -> JapanCbResult<()> {
    if let DataPointValue::Value(value) = data_point.value {
        if !value.is_finite() {
            return Err(JapanCbError::Invalid("观测值必须是有限数值".to_owned()));
        }
    }
    if data_point.metric_code.as_str().trim().is_empty() {
        return Err(JapanCbError::Missing("metric_code 为空".to_owned()));
    }
    if let DataPointValue::Missing(JapanCbMissingReason::SourceStatus(status)) = &data_point.value {
        if status.trim().is_empty() {
            return Err(JapanCbError::SemanticallyRejected(
                "缺失原因声明为源侧状态，但状态原文为空".to_owned(),
            ));
        }
    }
    // 频率与期间身份必须自洽：清单 §4 的期间取值没有「周」与「不规则」，
    // 故只有这两个频率允许与期间的粒度不一致。
    if !matches!(
        data_point.frequency,
        Frequency::Weekly | Frequency::Irregular
    ) && data_point.frequency != data_point.period.frequency()
    {
        return Err(JapanCbError::SemanticallyRejected(format!(
            "频率 {} 与期间 {} 的粒度不一致",
            data_point.frequency.name(),
            data_point.period.key()
        )));
    }
    Ok(())
}

/// 声明「单位换算 / 内容指纹 / 存储寻址」三类字段不属于本层。
///
/// 清单 §4 的 `value_usd` 是换算结果（换算归下游 Normalize）、`fingerprint` 需要
/// 哈希依赖且属派生、`raw_object_key` 是存储层寻址（属 ossx）。本层因此恒返回
/// [`JapanCbError::NotApplicable`]，避免把这三类语义夹带进源事实。
pub fn ensure_no_converted_or_derived_field() -> JapanCbResult<()> {
    Err(JapanCbError::NotApplicable(
        "value_usd（换算）/ fingerprint（派生哈希）/ raw_object_key（存储寻址）均不属于源事实层"
            .to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_no_converted_or_derived_field, validate_data_point, DataPointValue,
        JapanCbDataPoint, JapanCbMissingReason, Revision, Unit, SOURCE_UNIT_HUNDRED_MILLION_YEN,
    };
    use crate::error::{JapanCbErrorKind, JapanCbResult};
    use crate::value::date::Date;
    use crate::value::metric::{validate_metric_code, JP_BS_TOTAL_ASSETS};
    use crate::value::period::{Frequency, Period};

    fn unit() -> Unit {
        Unit::try_new(SOURCE_UNIT_HUNDRED_MILLION_YEN).expect("合法单位")
    }

    fn data_point(
        period: Period,
        frequency: Frequency,
        value: DataPointValue,
    ) -> JapanCbResult<JapanCbDataPoint> {
        JapanCbDataPoint::new(
            validate_metric_code(JP_BS_TOTAL_ASSETS).expect("合法 metric_code"),
            period,
            frequency,
            value,
            unit(),
            None,
        )
    }

    #[test]
    fn non_finite_constructor_is_rejected() {
        let period = Period::day(Date::new(2026, 9, 18).expect("合法日期"));
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                data_point(period, Frequency::Daily, DataPointValue::Value(value))
                    .expect_err("非有限值必须拒绝")
                    .kind(),
                crate::JapanCbErrorKind::Invalid
            );
        }
    }

    #[test]
    fn non_finite_mutation_is_rejected() {
        let period = Period::day(Date::new(2026, 9, 18).expect("合法日期"));
        let value = 1.0;
        let mut sample =
            data_point(period, Frequency::Daily, DataPointValue::Value(value)).expect("有限值合法");
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            sample.value = DataPointValue::Value(value);
            assert_eq!(
                validate_data_point(&sample)
                    .expect_err("修改后的非有限值必须拒绝")
                    .kind(),
                crate::JapanCbErrorKind::Invalid
            );
        }
    }

    #[test]
    fn unit_preserves_the_declared_source_unit() {
        let unit = Unit::try_new("  100 million yen  ").expect("合法单位");
        assert_eq!(unit.text(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
        assert_eq!(unit.to_string(), SOURCE_UNIT_HUNDRED_MILLION_YEN);
        assert!(Unit::try_new("   ").is_err());
    }

    #[test]
    fn revision_is_original_text_or_none() {
        let revision = Revision::try_new("  R1  ").expect("合法修订标识");
        assert_eq!(revision.text(), "R1");
        assert_eq!(revision.to_string(), "R1");
        assert!(Revision::try_new(" ").is_err());

        let point = data_point(
            Period::month(2026, 8).expect("合法期间"),
            Frequency::Monthly,
            DataPointValue::Value(1.0),
        )
        .expect("合法数据点");
        assert_eq!(point.revision, None, "无源事实时必须为 None");
    }

    #[test]
    fn missing_value_is_named_and_never_zero() {
        let point = data_point(
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            Frequency::Daily,
            DataPointValue::Missing(JapanCbMissingReason::SourceStatus(
                "SYNTH_STATUS".to_owned(),
            )),
        )
        .expect("合法数据点");
        assert_eq!(point.value.value(), None, "缺失不得转 0");
        assert!(point.value.is_missing());
        assert_eq!(
            point.value.missing_reason(),
            Some(&JapanCbMissingReason::SourceStatus(
                "SYNTH_STATUS".to_owned()
            ))
        );
        assert_eq!(
            JapanCbMissingReason::Unspecified.describe(),
            "源侧未给出状态标记"
        );
    }

    #[test]
    fn empty_source_status_is_rejected() {
        let result = data_point(
            Period::day(Date::new(2026, 9, 18).expect("合法日期")),
            Frequency::Daily,
            DataPointValue::Missing(JapanCbMissingReason::SourceStatus("  ".to_owned())),
        );
        assert_eq!(
            result.expect_err("空状态原文必须拒绝").kind(),
            JapanCbErrorKind::SemanticallyRejected
        );
    }

    #[test]
    fn frequency_must_match_period_granularity() {
        let mismatched = data_point(
            Period::year(2026).expect("合法期间"),
            Frequency::Daily,
            DataPointValue::Value(1.0),
        );
        assert_eq!(
            mismatched.expect_err("粒度不一致必须拒绝").kind(),
            JapanCbErrorKind::SemanticallyRejected
        );

        // 周 / 不规则没有对应的期间粒度，允许并存。
        assert!(data_point(
            Period::month(2026, 8).expect("合法期间"),
            Frequency::Weekly,
            DataPointValue::Value(1.0),
        )
        .is_ok());
    }

    #[test]
    fn data_point_id_composes_metric_period_and_revision() {
        let plain = data_point(
            Period::month(2026, 8).expect("合法期间"),
            Frequency::Monthly,
            DataPointValue::Value(1.0),
        )
        .expect("合法数据点");
        assert_eq!(plain.data_point_id(), "JP.BS.TOTAL_ASSETS:2026-08:none");

        let revised = JapanCbDataPoint::new(
            validate_metric_code(JP_BS_TOTAL_ASSETS).expect("合法 metric_code"),
            Period::month(2026, 8).expect("合法期间"),
            Frequency::Monthly,
            DataPointValue::Value(1.0),
            unit(),
            Some(Revision::try_new("R2").expect("合法修订标识")),
        )
        .expect("合法数据点");
        assert_eq!(revised.data_point_id(), "JP.BS.TOTAL_ASSETS:2026-08:R2");
        assert_ne!(plain.data_point_id(), revised.data_point_id());
    }

    #[test]
    fn ten_day_period_is_accepted_with_ten_day_frequency() {
        let point = data_point(
            Period::ten_day(2026, 9, 2).expect("合法旬期间"),
            Frequency::TenDay,
            DataPointValue::Value(1.0),
        )
        .expect("合法数据点");
        assert!(validate_data_point(&point).is_ok());
        assert_eq!(point.period.key(), "2026-09-T2");
    }

    #[test]
    fn converted_and_derived_fields_are_declared_out_of_scope() {
        assert_eq!(
            ensure_no_converted_or_derived_field()
                .expect_err("三类字段不属本层")
                .kind(),
            JapanCbErrorKind::NotApplicable
        );
    }
}
