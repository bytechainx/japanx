#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! japanx 热路径微基准：metric_code 校验 + 数据点解析。
//!
//! 入口为 `fn main()`，依赖 `Cargo.toml` 的 `[[bench]] harness = false`。
//! 输入为**合成文档**，不含任何真实源数据。

use std::hint::black_box;
use std::time::Instant;

use japanx::{parse_japanx_data_points, validate_metric_code, DECLARED_METRIC_CODES};

/// 合成文档：3 条数据点，字段名取自清单 §1.1 与 §4。
const DOCUMENT: &str = r#"{
  "_synthetic": true,
  "_note": "本文件为特性 005 自拟的合成样本，不是真实源数据，不构成任何证据。",
  "data_points": [
    {
      "metric_code": "JP.BS.TOTAL_ASSETS",
      "period": "2026-08",
      "frequency": "Monthly",
      "value": 1.0,
      "unit": "100 million yen",
      "revision": null
    },
    {
      "metric_code": "JP.IR.TONA",
      "period": "2026-09-18",
      "frequency": "Daily",
      "value": 0.5,
      "unit": "100 million yen"
    },
    {
      "metric_code": "JP.CA.EXCESS_RESERVES",
      "period": "2026-09-T2",
      "frequency": "TenDay",
      "value": null,
      "obs_status": "SYNTH_STATUS",
      "unit": "100 million yen"
    }
  ]
}"#;

fn main() {
    let iterations: u32 = 2_000;
    for _ in 0..10 {
        let _ = parse_japanx_data_points(DOCUMENT).expect("合成文档应可解析");
    }

    let start = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        let points = parse_japanx_data_points(DOCUMENT).expect("合成文档应可解析");
        checksum =
            checksum.wrapping_add(points.iter().map(|point| point.data_point_id().len()).sum());
        for code in DECLARED_METRIC_CODES {
            checksum = checksum.wrapping_add(
                validate_metric_code(code)
                    .expect("清单声明码应通过")
                    .as_str()
                    .len(),
            );
        }
        black_box(&points);
    }
    let elapsed = start.elapsed();

    println!(
        "bench_japanx_parse_offline: iters={iterations} total={elapsed:?} per_iter={:?} checksum={}",
        elapsed / iterations,
        black_box(checksum)
    );
}
