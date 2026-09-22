//! 期间与频率。
//!
//! 清单 `specs/adapter/japan_cb.md` §1.1/§4 用的期间集合是
//! `Daily|TenDay|Monthly|Quarterly|Annual`；公共形状契约
//! `source-library-contract.md` §2.2 要求的 `Period` 是
//! `Day|Month|Quarter|Year|Event` 五变体。
//!
//! 本库按「上级不可削弱，只可加严」同时满足两者：`Period` 是契约五变体的
//! **超集**（额外含清单要求的 **`TenDay`**），`Frequency` 是契约 §2.1 七取值的
//! 超集（额外含 `TenDay`）。契约既有变体一个未减。
//!
//! **禁止**用 `String` 顶替 `Period`。

use crate::error::{JapanCbError, JapanCbResult};
use crate::value::date::Date;

/// 频率。
///
/// 七取值与契约 `source-library-contract.md` §2.1 一致；额外含清单 §4 的
/// **十日（旬）** `TenDay`。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// 日频。
    Daily,
    /// 旬频（十日）。
    TenDay,
    /// 周频。
    Weekly,
    /// 月频。
    Monthly,
    /// 季频。
    Quarterly,
    /// 年频。
    Annual,
    /// 事件驱动。
    Event,
    /// 不规则。
    Irregular,
}

impl Frequency {
    /// 全部取值。
    pub const ALL: [Self; 8] = [
        Self::Daily,
        Self::TenDay,
        Self::Weekly,
        Self::Monthly,
        Self::Quarterly,
        Self::Annual,
        Self::Event,
        Self::Irregular,
    ];

    /// 清单 §4 声明的期间取值（逐字）。
    pub const SOURCE_DECLARED: [Self; 5] = [
        Self::Daily,
        Self::TenDay,
        Self::Monthly,
        Self::Quarterly,
        Self::Annual,
    ];

    /// 名称（本层自有拼写，非源侧 code list）。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::TenDay => "TenDay",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Annual => "Annual",
            Self::Event => "Event",
            Self::Irregular => "Irregular",
        }
    }

    /// 按名称解析（全部 8 个取值）。
    pub fn from_name(name: &str) -> JapanCbResult<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.name() == name)
            .ok_or_else(|| JapanCbError::Invalid(format!("未知频率：{name}")))
    }

    /// 按**清单声明的期间取值**解析。
    ///
    /// 只接受清单 §4 逐字列出的 5 个取值；`Weekly` / `Event` / `Irregular`
    /// 虽在契约取值域内，但清单未声明，故返回 `SemanticallyRejected`。
    pub fn from_source_token(token: &str) -> JapanCbResult<Self> {
        Self::SOURCE_DECLARED
            .into_iter()
            .find(|candidate| candidate.name() == token)
            .ok_or_else(|| {
                JapanCbError::SemanticallyRejected(format!(
                    "清单未声明该期间取值：{token}（清单只有 Daily|TenDay|Monthly|Quarterly|Annual）"
                ))
            })
    }
}

/// 业务期间。
///
/// 契约 §2.2 的五变体（`Day` / `Month` / `Quarter` / `Year` / `Event`）之外，
/// 额外含清单 §1.1/§4 要求的 **`TenDay`**（旬），故为契约的超集。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// 某日。
    Day(Date),
    /// 某年某月某旬。
    ///
    /// `segment` ∈ 1–3（上旬 / 中旬 / 下旬）。清单只给出 `TenDay` 这个名字，
    /// 未定义旬的边界，故本库只表达「该年该月的第几旬」这一**身份**，
    /// 不承载边界语义。
    TenDay {
        /// 年。
        year: i16,
        /// 月（1–12）。
        month: u8,
        /// 旬序号（1–3）。
        segment: u8,
    },
    /// 某年某月。
    Month {
        /// 年。
        year: i16,
        /// 月（1–12）。
        month: u8,
    },
    /// 某年某季度。
    Quarter {
        /// 年。
        year: i16,
        /// 季（1–4）。
        quarter: u8,
    },
    /// 某年。
    Year(i16),
    /// 某事件日。
    Event {
        /// 事件日期。
        date: Date,
    },
}

impl Period {
    /// 构造「某日」。
    #[must_use]
    pub fn day(date: Date) -> Self {
        Self::Day(date)
    }

    /// 构造「某年某月某旬」；月或旬序号非法时返回 `Invalid`。
    pub fn ten_day(year: i16, month: u8, segment: u8) -> JapanCbResult<Self> {
        Date::new(year, month, 1).map_err(|error| match error {
            JapanCbError::Invalid(message) => {
                JapanCbError::Invalid(format!("旬期间非法：{message}"))
            }
            other => other,
        })?;
        if !(1..=3).contains(&segment) {
            return Err(JapanCbError::Invalid(format!("旬序号 {segment} 超出 1–3")));
        }
        Ok(Self::TenDay {
            year,
            month,
            segment,
        })
    }

    /// 构造「某年某月」；月份非法时返回 `Invalid`。
    pub fn month(year: i16, month: u8) -> JapanCbResult<Self> {
        Date::new(year, month, 1).map_err(|error| match error {
            JapanCbError::Invalid(message) => {
                JapanCbError::Invalid(format!("月期间非法：{message}"))
            }
            other => other,
        })?;
        Ok(Self::Month { year, month })
    }

    /// 构造「某年某季度」；季度非法时返回 `Invalid`。
    pub fn quarter(year: i16, quarter: u8) -> JapanCbResult<Self> {
        if !(1..=4).contains(&quarter) {
            return Err(JapanCbError::Invalid(format!("季度 {quarter} 超出 1–4")));
        }
        Date::new(year, (quarter - 1) * 3 + 1, 1)?;
        Ok(Self::Quarter { year, quarter })
    }

    /// 构造「某年」；年份非法时返回 `Invalid`。
    pub fn year(year: i16) -> JapanCbResult<Self> {
        Date::new(year, 1, 1)?;
        Ok(Self::Year(year))
    }

    /// 构造「某事件日」。
    #[must_use]
    pub fn event(date: Date) -> Self {
        Self::Event { date }
    }

    /// 规范键（可被 [`Period::parse`] 反向解析）。
    ///
    /// 形态：`YYYY-MM-DD`（日）/ `YYYY-MM-Tn`（旬）/ `YYYY-MM`（月）/
    /// `YYYY-Qn`（季）/ `YYYY`（年）/ `event:YYYY-MM-DD`（事件）。
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::Day(date) => date.to_iso(),
            Self::TenDay {
                year,
                month,
                segment,
            } => format!("{year:04}-{month:02}-T{segment}"),
            Self::Month { year, month } => format!("{year:04}-{month:02}"),
            Self::Quarter { year, quarter } => format!("{year:04}-Q{quarter}"),
            Self::Year(year) => format!("{year:04}"),
            Self::Event { date } => format!("event:{}", date.to_iso()),
        }
    }

    /// 严格解析 [`Period::key`] 产生的形态。
    pub fn parse(input: &str) -> JapanCbResult<Self> {
        if let Some(rest) = input.strip_prefix("event:") {
            return Ok(Self::event(Date::parse(rest)?));
        }
        let bytes = input.as_bytes();
        match bytes.len() {
            4 => Self::year(parse_year(input)?),
            7 if bytes[4] == b'-' && bytes[5] == b'Q' => {
                Self::quarter(parse_year(&input[0..4])?, parse_u8(&input[6..7], "季度")?)
            }
            7 if bytes[4] == b'-' => {
                Self::month(parse_year(&input[0..4])?, parse_u8(&input[5..7], "月份")?)
            }
            10 if bytes[4] == b'-' && bytes[7] == b'-' && bytes[8] == b'T' => Self::ten_day(
                parse_year(&input[0..4])?,
                parse_u8(&input[5..7], "月份")?,
                parse_u8(&input[9..10], "旬序号")?,
            ),
            10 => Ok(Self::day(Date::parse(input)?)),
            _ => Err(JapanCbError::Invalid(format!(
                "期间形态非法：{input}（须为 YYYY-MM-DD / YYYY-MM-Tn / YYYY-MM / YYYY-Qn / YYYY / event:YYYY-MM-DD）"
            ))),
        }
    }

    /// 与该期间最贴合的频率。
    #[must_use]
    pub fn frequency(&self) -> Frequency {
        match self {
            Self::Day(_) => Frequency::Daily,
            Self::TenDay { .. } => Frequency::TenDay,
            Self::Month { .. } => Frequency::Monthly,
            Self::Quarter { .. } => Frequency::Quarterly,
            Self::Year(_) => Frequency::Annual,
            Self::Event { .. } => Frequency::Event,
        }
    }
}

/// 解析固定 4 位年份。
fn parse_year(segment: &str) -> JapanCbResult<i16> {
    if segment.len() != 4 || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(JapanCbError::Invalid(format!("年份形态非法：{segment}")));
    }
    segment
        .parse::<i16>()
        .map_err(|error| JapanCbError::Invalid(format!("年份无法解析：{error}")))
}

/// 解析固定数字段。
fn parse_u8(segment: &str, label: &str) -> JapanCbResult<u8> {
    if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(JapanCbError::Invalid(format!("{label}形态非法：{segment}")));
    }
    segment
        .parse::<u8>()
        .map_err(|error| JapanCbError::Invalid(format!("{label}无法解析：{error}")))
}

#[cfg(test)]
mod tests {
    use super::{Frequency, Period};
    use crate::error::JapanCbErrorKind;
    use crate::value::date::Date;

    fn date(year: i16, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("合法日期")
    }

    #[test]
    fn frequency_names_round_trip() {
        for frequency in Frequency::ALL {
            assert_eq!(
                Frequency::from_name(frequency.name()).expect("可回读"),
                frequency
            );
        }
        assert_eq!(Frequency::ALL.len(), 8);
        assert_eq!(Frequency::SOURCE_DECLARED.len(), 5);
        assert!(Frequency::from_name("Fortnightly").is_err());
    }

    #[test]
    fn source_tokens_are_limited_to_the_declared_five() {
        for frequency in Frequency::SOURCE_DECLARED {
            assert_eq!(
                Frequency::from_source_token(frequency.name()).expect("清单声明值"),
                frequency
            );
        }
        for undeclared in ["Weekly", "Event", "Irregular", "Tenday", ""] {
            let error = Frequency::from_source_token(undeclared).expect_err(undeclared);
            assert_eq!(error.kind(), JapanCbErrorKind::SemanticallyRejected);
        }
    }

    #[test]
    fn period_keys_round_trip() {
        let cases = [
            Period::day(date(2026, 9, 18)),
            Period::ten_day(2026, 9, 2).expect("合法旬期间"),
            Period::month(2026, 9).expect("合法月期间"),
            Period::quarter(2026, 3).expect("合法季期间"),
            Period::year(2026).expect("合法年期间"),
            Period::event(date(2026, 9, 18)),
        ];
        for period in cases {
            let key = period.key();
            assert_eq!(Period::parse(&key).expect("可回读"), period, "key={key}");
        }
    }

    #[test]
    fn period_keys_have_fixed_shapes() {
        assert_eq!(Period::day(date(2026, 9, 18)).key(), "2026-09-18");
        assert_eq!(
            Period::ten_day(2026, 9, 2).expect("合法").key(),
            "2026-09-T2"
        );
        assert_eq!(Period::month(2026, 9).expect("合法").key(), "2026-09");
        assert_eq!(Period::quarter(2026, 3).expect("合法").key(), "2026-Q3");
        assert_eq!(Period::year(2026).expect("合法").key(), "2026");
        assert_eq!(Period::event(date(2026, 9, 18)).key(), "event:2026-09-18");
    }

    #[test]
    fn period_frequency_covers_the_declared_granularities() {
        assert_eq!(Period::day(date(2026, 9, 18)).frequency(), Frequency::Daily);
        assert_eq!(
            Period::ten_day(2026, 9, 2).expect("合法").frequency(),
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
        assert_eq!(
            Period::event(date(2026, 9, 18)).frequency(),
            Frequency::Event
        );
    }

    #[test]
    fn period_parse_rejects_malformed_input() {
        let cases = [
            "2026-2-3",
            "2026/09/18",
            "2026-13",
            "2026-Q5",
            "2026-Q0",
            "2026-09-T0",
            "2026-09-T4",
            "2026-09-T",
            "26",
            "",
        ];
        for input in cases {
            let error = Period::parse(input).expect_err(input);
            assert_eq!(error.kind(), JapanCbErrorKind::Invalid, "{input}");
        }
    }

    #[test]
    fn period_constructors_validate_ranges() {
        assert!(Period::month(2026, 0).is_err());
        assert!(Period::month(2026, 13).is_err());
        assert!(Period::quarter(2026, 0).is_err());
        assert!(Period::quarter(2026, 5).is_err());
        assert!(Period::year(0).is_err());
        assert!(Period::ten_day(2026, 9, 0).is_err());
        assert!(Period::ten_day(2026, 9, 4).is_err());
        assert!(Period::ten_day(2026, 13, 1).is_err());
        assert!(Period::ten_day(2026, 9, 3).is_ok());
    }
}
