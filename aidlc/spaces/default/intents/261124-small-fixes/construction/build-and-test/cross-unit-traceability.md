# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 古い説明を直す | BR1.1、BR1.2 | `decide_before_run`(`flow/decide.rs` の外)は 0 件。runtime の再公開の説明も実物に合わせた |
| SM2 | 仕様書 rust 02 を実装に合わせる | BR2.1、BR2.2、BR2.3、BR2.4 | 型と一覧を直し、3 段落の版の注記を「今の形」と「版の履歴」に分けた。rust 05・06 も直した |
| SM3 | テストの重複を消す | BR3.1、BR3.2、BR3.3、BR3.4、BR3.5 | 3 件を消し、1 件を移して `plan_perf.rs` をなくした。`signals_order.rs` は残した。言語に依らない 11 章の根拠の欄を直した |
| SM4 | ソースの検査を強める | BR4.1、BR4.2 | include の 3 つのマクロも止める。今のコードで通り、足すと失敗することを確かめた。既知の限界をコメントに書いた |
| SM5 | 件数とテスト名の差 | BR9.2、BR9.3 | 1,152 − 3 + 0 = 1,149。名前の差は消した 3 件だけ |
| SM6 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 本番の変更は説明の文だけ。版は 0.9.8 のまま |
| SM7 | 合格の条件 | BR9.4 | fmt、clippy(本番の形と Windows を含む)、1,149 通過、行カバレッジ 98.06%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
