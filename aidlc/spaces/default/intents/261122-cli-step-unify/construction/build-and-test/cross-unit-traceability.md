# ユニットをまたぐ追跡(Cross-Unit Traceability): run と pipeline の重複をなくすリファクタリング

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、成功の指標(SM1〜SM4)と、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| ID | 内容 | 実装 | テスト |
|----|------|------|--------|
| SM1、BR1.1 | 1 つのジョブの共通の手順を 1 か所にまとめる | `crates/cli/src/step.rs`、`run.rs`、`pipeline.rs` | 既存の流れのテスト |
| BR1.2 | 消す処理は共通、いつ消すかは呼び出し側 | `step.rs`(`remove_if_requested`)、`run.rs`、`pipeline.rs` | `flow_run.rs`、`flow_pipeline.rs`(新規 3 本) |
| SM2、BR2.1 | 動きは変わらない | (順序、Durability、文脈を移したまま) | 既存の cli のテスト 440 件(無変更) |
| SM3、BR3.1 | 仕様書の実行の流れを共通の手順として書き直す | `docs/spec/rust/05-execution-flow.md`、`02-architecture.md` | (読み合わせ。確認で一致を確認) |
| SM4、BR4.1 | 合格の条件 | 全体 | fmt、clippy、全テスト、カバレッジ 97.99% |

覆われていない規則・指標: なし。
