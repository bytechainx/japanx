//! japanx 的错误分类与错误类型。
//!
//! 分类按「调用方应如何反应」划分：授权拒绝、路由拒绝、越权写入与语义拒绝
//! 各有独立变体，调用方无需做字符串匹配即可分流。

/// 错误分类：按「调用方应如何反应」划分。
///
/// 禁止用字符串匹配替代对本枚举的匹配。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JapanCbErrorKind {
    /// 输入形态或取值非法（调用方应修数据，重试无意义）。
    Invalid,
    /// 缺少必需项。
    Missing,
    /// 授权判定未通过（fail-closed 落点）。
    AuthorizationDenied,
    /// 该产品不属于本域，已被路由规则拒绝。
    RoutedElsewhere,
    /// 越权写入（权威写入归他域）。
    WriteAuthorityDenied,
    /// 结构可解析但语义不被接受（如清单未声明的 metric_code / 频率）。
    SemanticallyRejected,
    /// 尚未实现的规划能力。
    NotApplicable,
    /// 不变量被破坏（库内 bug 的信号）。
    Invariant,
}

/// japanx 错误。保留可区分的分类与来源链。
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum JapanCbError {
    /// 输入非法。
    #[error("输入非法：{0}")]
    Invalid(String),
    /// 缺少必需项。
    #[error("缺少必需项：{0}")]
    Missing(String),
    /// 授权判定未通过。
    #[error("授权判定未通过：{0}")]
    AuthorizationDenied(String),
    /// 产品不属于本域。
    #[error("不属于本域，已路由他处：{0}")]
    RoutedElsewhere(String),
    /// 越权写入。
    #[error("越权写入被拒绝：{0}")]
    WriteAuthorityDenied(String),
    /// 语义拒绝。
    #[error("语义不被接受：{0}")]
    SemanticallyRejected(String),
    /// 规划能力未实现。
    #[error("规划能力尚未实现：{0}")]
    NotApplicable(String),
    /// 不变量被破坏。
    #[error("不变量被破坏：{0}")]
    Invariant(String),
}

impl JapanCbError {
    /// 分类，供调用方按「如何反应」分流。
    #[must_use]
    pub fn kind(&self) -> JapanCbErrorKind {
        match self {
            Self::Invalid(_) => JapanCbErrorKind::Invalid,
            Self::Missing(_) => JapanCbErrorKind::Missing,
            Self::AuthorizationDenied(_) => JapanCbErrorKind::AuthorizationDenied,
            Self::RoutedElsewhere(_) => JapanCbErrorKind::RoutedElsewhere,
            Self::WriteAuthorityDenied(_) => JapanCbErrorKind::WriteAuthorityDenied,
            Self::SemanticallyRejected(_) => JapanCbErrorKind::SemanticallyRejected,
            Self::NotApplicable(_) => JapanCbErrorKind::NotApplicable,
            Self::Invariant(_) => JapanCbErrorKind::Invariant,
        }
    }

    /// 是否值得重试。本层无网络，除 `Invariant` 外一律 `false`。
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind(), JapanCbErrorKind::Invariant)
    }
}

/// 本 crate 统一结果别名。
pub type JapanCbResult<T> = Result<T, JapanCbError>;

#[cfg(test)]
mod tests {
    use super::{JapanCbError, JapanCbErrorKind};

    fn samples() -> Vec<JapanCbError> {
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

    #[test]
    fn kind_maps_every_variant_distinctly() {
        let kinds: Vec<JapanCbErrorKind> = samples().iter().map(JapanCbError::kind).collect();
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
        let key = [
            JapanCbErrorKind::AuthorizationDenied,
            JapanCbErrorKind::RoutedElsewhere,
            JapanCbErrorKind::WriteAuthorityDenied,
            JapanCbErrorKind::SemanticallyRejected,
        ];
        for (index, left) in key.iter().enumerate() {
            for right in &key[index + 1..] {
                assert_ne!(left, right, "关键拒绝分类不得互相混同");
            }
        }
    }

    #[test]
    fn only_invariant_is_retryable() {
        for error in samples() {
            assert_eq!(
                error.is_retryable(),
                error.kind() == JapanCbErrorKind::Invariant,
                "{error:?}"
            );
        }
    }

    #[test]
    fn display_is_non_empty_for_every_variant() {
        for error in samples() {
            assert!(!error.to_string().is_empty(), "{error:?}");
            assert!(!format!("{error:?}").is_empty(), "{error:?}");
        }
    }

    #[test]
    fn messages_do_not_echo_credentials_or_source_lines() {
        let error = JapanCbError::SemanticallyRejected("清单未声明该 metric_code".to_owned());
        let text = error.to_string();
        assert!(text.contains("清单未声明该 metric_code"));
        assert!(!text.contains("password"), "{text}");
        assert!(!text.contains("token"), "{text}");
        assert!(!text.contains('='), "错误消息不得回显配置源码行：{text}");
    }
}
