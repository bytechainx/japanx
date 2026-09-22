//! japanx 的离线解析门面。
//!
//! 两个入口，都只接受**文本**，不接受 URL、HTTP 客户端或认证参数：
//!
//! | 入口 | 输入形态 | 产出 |
//! | --- | --- | --- |
//! | [`parse_japanx_data_points`] | JSON 文档 | 数据点集合 |
//! | [`parse_japanx_source_definition`] | TOML 文档 | 源定义 |

mod document;
mod source_definition;

pub use document::parse_japanx_data_points;
pub use source_definition::{
    parse_japanx_source_definition, JapanCbSourceDefinition, PARSER_INPUT_ENCODING,
    SOURCE_ENCODING_SHIFT_JIS, SOURCE_KIND_CSV_DOWNLOAD,
};

#[cfg(test)]
mod tests {
    use super::{
        parse_japanx_data_points, parse_japanx_source_definition, SOURCE_KIND_CSV_DOWNLOAD,
    };

    #[test]
    fn both_entries_work_offline_on_synthetic_text() {
        let document = r#"{ "_synthetic": true, "_note": "合成样本",
            "data_points": [ { "metric_code": "JP.MB.TOTAL", "period": "2026-08",
              "frequency": "Monthly", "value": 1.0, "unit": "100 million yen" } ] }"#;
        let points = parse_japanx_data_points(document).expect("合成文档应可解析");
        assert_eq!(points.len(), 1);

        let toml = "kind = \"csv_download\"\nencoding = \"shift_jis\"\nunit = \"100 million yen\"\nscale = 1\nmetric_codes = [\"JP.MB.TOTAL\"]\n";
        let definition = parse_japanx_source_definition(toml).expect("合法源定义");
        assert_eq!(definition.kind(), SOURCE_KIND_CSV_DOWNLOAD);
    }
}
