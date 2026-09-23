# japanx Agent 指南

> 本文件为 AI Agent 在本仓库工作时的入口指南。

## 项目定位

日本央行（BOJ）源事实的**类型层**：12 个 `JP.*` metric_code、期间与频率、源侧单位、
离线解析、校验与 fail-closed 授权判定。不做联网采集、不做认证、不做缓存、不做再分发、
不做存储、不做单位换算、不做派生指标、不做编码转码。

## 技术栈

- Rust edition 2021，`rust-version = 1.85`
  （按依赖图 `rust_version` 最大值推导，由 `toml` → `toml_edit` → `indexmap` → `hashbrown 0.17` 驱动，见 `CONTEXT.md`）
- 关键依赖：`thiserror` 2、`serde` 1（derive）、`serde_json` 1、`toml` 0.8
- **无 path 依赖**：不依赖 `instrumentationx` 或任何 `bytechainx/*` crate
- crate 级 lint：`unwrap_used` / `expect_used` / `panic` / `unreachable` / `todo` /
  `unimplemented` 全部 `deny`（测试代码经 `cfg_attr(test)` 豁免）

## 代码结构

```text
src/
├── lib.rs                 # crate 文档 + 模块声明 + 门面 pub use
├── error.rs               # JapanCbError / JapanCbErrorKind / JapanCbResult
├── authz.rs               # JapanCbAuthorization / 证据登记 / fail-closed 判定
├── pit.rs                 # publication 语义三元组（恒 Date + Inferred + NotEligible）
├── parse.rs               # 解析门面（JSON 数据点 + TOML 源定义）
│   ├── parse/document.rs          # 数据点 JSON 解析
│   └── parse/source_definition.rs # TOML 源定义解析
└── value.rs               # 值对象门面
    ├── value/date.rs         # Date
    ├── value/period.rs       # Period / Frequency
    ├── value/metric.rs       # 12 个 metric_code / 类别 / 跨源守卫
    └── value/datapoint.rs    # Unit / Revision / 值 / JapanCbDataPoint
```

- 依赖方向单向：`error ← value ← {authz, pit, parse}`，无环
- `#![forbid(unsafe_code)]`、`#![deny(missing_docs)]`、`#![deny(unreachable_pub)]`

## 硬约束（改代码前先读）

| 禁止 | 原因 |
| --- | --- |
| 写任何 `https://…` 端点字面量、Header、凭据字段 | 清单把统计码 / URL / 许可登记为 UNKNOWN |
| **写具体限流数字**（如 `rate_limit_rps = 5`） | 授权前不写限流值；源定义模型不含该字段，写了会被原子拒绝 |
| 编造统计码全集或官方 stat_code 字典 | 清单只有 12 个 metric_code |
| 引入 HTTP 客户端 / 异步运行时 / `chrono` / `time` / `rand` / 编码库 | `FR-037`、`MR-DATA-003`；转码归采集侧 |
| 读环境变量或凭据 | 本层只做离线解析 |
| 主张 `JPY=X` / `USDJPY=X` 为主源 | 归 `fredx` 独占（`DEXJPUS`） |
| 把 MOF / BOJ 曲线晋级 | 归 `yieldx` #13–14 |
| 在值对象里实现单位换算或派生指标 | `value_usd` / `fingerprint` / `raw_object_key` 均不属本层 |
| 把合成夹具写成「实测」「核验 PASS」 | 伪造证据（`FR-057`） |
| 建 `mod.rs` 或名为 `utils`/`helpers`/`common`/`manager`/`base`/`global`/`misc` 的文件 | `MR-STRUCT-003` / `MR-STRUCT-005` |
| 改 `specs/`、`docs/` 之外的共享文件或其它 crate 目录 | 越界写入 |

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

## 相关文档

- 组织 Rust 规范：`~/org-config/rulesets/rust/RULES.md`
- 公共形状契约：`specs/features/005-macro-data-source-crates/contracts/source-library-contract.md`
- 跨源语义：`specs/features/005-macro-data-source-crates/contracts/cross-source-routing.md`
- 采集范围权威：`specs/adapter/japan_cb.md`
- API 文档：`docs/API.md`
- 标准与验收：`docs/标准.md`
- 术语与边界：`CONTEXT.md`
- 变更记录：`CHANGELOG.md`
