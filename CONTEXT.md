# japanx 上下文

本文件定义 `japanx` 与其使用方共享的核心词汇与边界。它只记录领域含义与能力边界，
不记录具体实现、存储或部署决定。

## 角色与边界

**源事实类型层**：把 `specs/adapter/japan_cb.md` 已钉死的 metric_code、期间与单位
落成可机械校验的 Rust 类型，并提供离线解析与 fail-closed 授权判定。
_Avoid_: BOJ 采集器 / SDK（联网采集、认证、缓存、再分发都不在本层）

**离线**：只处理**字符串 → 值对象**，不持有任何网络能力、不读凭据与环境变量。
_Avoid_: 采集器（本层不发起任何请求）

**授权判定是只读结论**：判定「该源的这个范围是否被授权访问」，不改写清单或名册的任何登记值。
_Avoid_: 授权变更（本特性不改变任何授权状态）

## 核心词汇

**metric_code**：清单 §1.1 的规范形 `JP.<类别>.<指标>[.<子维度>]`，声明集为 12 个。
_Avoid_: stat_code（清单**没有**官方 stat_code 字典，本库不编造该全集）

**类别码**（`MetricCategory`）：`BS` / `IR` / `MB` / `MS` / `CA` / `MO` / `FX` / `TK`，
由 12 个 metric_code 的第三段推得。
_Avoid_: 产品分组（类别只用于 shape 校验，不代表产品树）

**期间**（`Period`）：契约 §2.2 五变体的**超集**——额外含清单 §4 的旬 `TenDay`。
_Avoid_: 时间戳（本层只表达期间身份，不表达时刻）

**频率**（`Frequency`）：契约 §2.1 七取值的**超集**——额外含 `TenDay`。
清单声明的取值只有 5 个（`Daily|TenDay|Monthly|Quarterly|Annual`），
故 `from_source_token` 只接受这 5 个。
_Avoid_: 采样率（频率是源侧发布节律的声明，不是采样率）

**源侧单位**（`Unit`）：清单 §1.2/§3 的 `100 million yen`（日元亿），原样保留。
_Avoid_: 归一化单位 / `value_usd`（换算归下游 Normalize，本层不做）
_Avoid_: 请勿把 `Unit` 当作可枚举集合——清单只给了一个单位，凭空虚造集合就是编造

**修订标识**（`Revision`）：清单 §4 的 `revision` 字段原文；清单未定义其形态，
故本层只原样承载，不解释、不派生。
_Avoid_: vintage 面（无官方 vintage 面时整字段为 `None`，不得伪造）

**具名缺失**（`JapanCbMissingReason`）：缺失值必须带原因，绝不静默转 0。
_Avoid_: 空值 / 0 值

**源定义**（`JapanCbSourceDefinition`）：TOML 驱动的离线形态，字段面只含清单已钉死的
`kind` / `encoding` / `unit` / `scale` / `metric_codes`。
_Avoid_: 采集配置（本层不承载限流、调度、端点等采集侧参数）

**格式契约**：源侧声明 `encoding = shift_jis`，本层解析入口要求**已转码的 UTF-8 文本**；
本库**不实装任何编码库**。
_Avoid_: 转码器（转码归采集侧）

## rust-version 推导

按「依赖图中所有依赖所声明 `rust-version` 的最大值」推导
（`cargo metadata --format-version 1` 的 `rust_version` 字段）。实测（2026-09-22）：

| 依赖 | 版本 | 其 `rust-version` |
| --- | --- | --- |
| `equivalent` | 1.0.2 | 1.6 |
| `memchr` | 2.8.3 | 1.61 |
| `serde` / `serde_core` | 1.0.229 | 1.56 |
| `winnow` | 0.7.15 | 1.65.0 |
| `serde_spanned` / `toml` / `toml_datetime` / `toml_edit` / `toml_write` | 见 `cargo metadata` | 1.66 |
| `itoa` | 1.0.18 | 1.68 |
| `proc-macro2` / `quote` / `serde_derive` / `serde_json` / `syn` / `thiserror` / `thiserror-impl` / `unicode-ident` / `zmij` | 见 `cargo metadata` | 1.71 |
| **`hashbrown`** | 0.17.1 | **1.85.0** |
| `indexmap` | 2.14.2 | 1.85 |

最大值 = **1.85**，故 `Cargo.toml` 声明 `rust-version = "1.85"`。
**该下界由 `toml` 依赖链驱动**（`toml` → `toml_edit` → `indexmap` → `hashbrown 0.17`）；
若将来 `toml` 换成不依赖 `indexmap` 的版本，须重新推导。
无依赖缺失 `rust_version` 的情形，无需保守取值。

## 已知缺口

1. **无 Owner 签核文件**：`authorization = unknown`，授权判定恒 `Denied`；
   本库不假装有证据，也不提供任何「放行」运行时路径。
2. **无官方 stat_code 字典**：清单只声明 12 个 metric_code，
   `validate_metric_code` 只校验规范形，`is_declared_metric_code` 判定是否在声明集内。
3. **Shift-JIS / HTML 解析栈未落地**（清单登记 `absent`）：本库不实装编码库、
   不解析 HTML；只接受已转码的 UTF-8 文本。
4. **live 12 指标未实现且被阻断**（`blocked`）：无 HTTP 依赖、无端点字面量、
   Header 与限流数字一律不写。
5. **主权未决**：JGB 10Y / 日本 CPI 的「主权 vs 转发」待裁，
   `claim_forwarded_series_sovereignty` 恒拒绝；MOF / BOJ 曲线归 `yieldx` #13–14。
6. **清单 §4 的三个字段有意排除**：`value_usd`（换算）、`fingerprint`（派生哈希）、
   `raw_object_key`（存储寻址），见 `ensure_no_converted_or_derived_field`。
7. **名称占用**：`japanx` 未发布到 crates.io，仅以 git / path 依赖使用。
