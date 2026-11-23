# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 削除と公開範囲の縮小 | BR1.1〜BR1.6、BR2.1〜BR2.6 | テストだけが使う実装(`From`、`as_str`、`is_raised`・`==`、`Default` 4 つ、`sanitize`、`reason()`)を消し、公開範囲を使う範囲に狭めた。消す・狭める名前の参照は 0 件 |
| SM2 | `RunIdError` を 1 つに | BR3.1 | 単位構造体にした。単体テストで 4 つの場合の入力が通らないことを確かめる |
| SM3 | runtime の core の再公開をやめる | BR4.1 | `pub use conductor_core` は 0 件。使う所は `conductor_core` から使う |
| SM4 | 種類の `Display`・`Error`、使われない `Hash` | BR5.1、BR5.2 | 消した。本番で使う所は無い |
| SM5 | 誤りの型の `Display` と `Error` は残す | BR5.3 | 残した。表示の安全のテストは期待を変えずに通る |
| SM6 | 確かめる手段 | BR9.2、BR9.3、BR9.6 | §7 の検索、テスト名の比較、`cargo doc` の警告(5 件のまま)を走らせた |
| SM7 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 期待の文字列は無変更で通る。版は 0.9.8 のまま |
| SM8 | 既存のテストの扱いと件数 | BR8.1、BR9.2、BR9.3 | テストの変更は T1〜T32、消したテストは §4.6 の 4 件。1,157 − 4 + 0 = 1,153 |
| SM9 | 合格の条件 | BR9.4 | fmt、clippy、1,153 通過、行カバレッジ 98.08%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI で通った |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
