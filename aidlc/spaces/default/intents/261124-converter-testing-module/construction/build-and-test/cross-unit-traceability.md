# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | `testing` と `opposite` を本番から消す | BR1.1、BR1.2 | 消した。参照は 0 件 |
| SM2 | 木を比べるテストを単体テストへ移す | BR3.1、BR3.2、BR3.3 | 45 件を `src/tests/` へ移した。名前、確かめの行、期待の値は変わらない。BR3.2 の置き場所の制約を守った |
| SM3 | `PartialEq` を `#[cfg(test)]` の中だけに | BR2.1 | 本番の形(`--lib`)のビルドと clippy が通る |
| SM4 | 仕様書 rust を実装に合わせる | BR4.1、BR4.2 | 02・09・11 章を直した。`testing`・`read_tree`・`opposite` は 0 件。言語に依らない仕様書は変えない(D4) |
| SM5 | 確かめる手段 | BR9.2、BR9.3、BR9.6 | §7 の検索、テスト名の比較、`cargo doc` の警告(5 件のまま)、ソースの検査の範囲の記録 |
| SM6 | 動きを変えない、版を上げない | BR9.1、BR9.5 | cli の `convert` のテストを含め、期待の文字列は無変更で通る。版は 0.9.8 のまま |
| SM7 | 既存のテストの扱いと件数 | BR8.1、BR9.2、BR9.3 | 移したのは 45 件、消したのは 1 件(D3)。1,153 − 1 + 0 = 1,152 |
| SM8 | 合格の条件 | BR9.4 | fmt、clippy、1,152 通過、行カバレッジ 98.09%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
