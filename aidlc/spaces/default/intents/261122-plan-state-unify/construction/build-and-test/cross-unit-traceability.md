# ユニットをまたぐ追跡(Cross-Unit Traceability): `--dry-run` の模擬の重複をなくすリファクタリング

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、成功の指標(SM1〜SM5)と、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| ID | 内容 | 実装 | テスト |
|----|------|------|--------|
| SM1、BR1.1、BR1.2 | 更新の関数を core に置き、runtime は再エクスポート | `crates/core/src/flow/record.rs`、`crates/runtime/src/lib.rs` | `record.rs` の単体のテスト(移動、中身は無変更) |
| SM2、BR1.3 | 模擬も本番と同じ記録 | `crates/core/src/plan/simulate.rs` | 比べる確認(90 万例、違い 0)、`plan_pipeline.rs`(承認した形) |
| SM3、BR2.1、BR2.2 | 動きは変わらない | (順序と規則は同じ) | 既存のテスト(無変更)、比べる確認 |
| BR2.3 | `apply_skip` を O(n) | `record.rs` | `plan_perf.rs`(新規) |
| SM4、BR3.1 | 仕様書 | `docs/spec/rust` の 05、02、12 | (読み合わせ。確認で一致を確認) |
| SM5、BR4.1 | 合格の条件 | 全体 | fmt、clippy、全テスト、カバレッジ 97.96% |

覆われていない規則・指標: なし。
