# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | `retry.rs` を `step.rs` へ | BR1.1、BR1.2 | 移した。`run_job_with_retry` は非公開で `run_step` の直後。`retry.rs` は無い |
| SM2 | `report/escape.rs` を `report/mod.rs` へ | BR2.1、BR2.2 | 移した。2 つの関数は `pub(crate)` で `push_line` の直前。`report/escape.rs` は無い |
| SM3 | `graph`・`init` を `aux_commands.rs` へ | BR3.1、BR3.2 | 移した。`graph.rs`・`init.rs` は無い。見出しに分け方の基準を書いた |
| SM4 | `platform` の中継を 1 段に | BR4.1、BR4.2 | runtime と同じ再公開の形。ファイル名は変わらない。Windows の対象の clippy が通る |
| SM5 | 無くなったものの検索 | BR5.2 | ファイルとモジュールのパスの参照は 0 件 |
| SM6 | テストは `use` の書き換えだけ | BR9.2、BR9.3 | T1 の 1 行だけ。テスト名の差 0、件数 1,149 のまま |
| SM7 | ソースの検査 | BR4.2、BR9.6 | cli の `nfr_source` 6 件が通る |
| SM8 | 仕様書 rust | BR5.1、BR5.2 | 24 か所を直した。古い名前は残らない |
| SM9 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 処理のコードは変えていない。版は 0.9.8 のまま |
| SM10 | 合格の条件 | BR9.4 | fmt、clippy(本番の形と Windows の対象を含む)、1,149 通過、行カバレッジ 98.06%。依存の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
