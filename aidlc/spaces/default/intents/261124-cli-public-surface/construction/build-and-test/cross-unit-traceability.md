# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 公開範囲を main と継ぎ目と要る型だけに | BR1.1、BR1.2、BR1.3、BR2.1 | 残す 13 項目と一致、15 項目を狭めた。`tests/` から狭めた名前の参照は 0 件 |
| SM2 | 狭めた項目を呼ぶテストを単体テストへ | BR2.1、BR3.1、BR3.2、BR3.3、BR3.4 | 162 件を `src/tests/` へ移した。名前・確かめの行・期待の値は不変。ソースの検査は D2 の範囲で通る |
| SM3 | 仕様書 rust を実装に合わせる | BR4.1、BR4.2 | 02・07・08・11 章を直した。言語に依らない仕様書は変えない(D6) |
| SM4 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 本番の処理は不変。期待の文字列は無変更で通る。版は 0.9.8 のまま |
| SM5 | 既存のテストの扱いと件数 | BR3.3、BR8.1、BR9.2、BR9.3、BR9.6 | 移したのは 162 件、書き方を変えたのは T8〜T10。消したテスト・足したテストは 0。1,152 のまま |
| SM6 | 合格の条件 | BR1.3、BR9.4 | fmt、clippy(本番の形と Windows を含む)、1,152 通過、行カバレッジ 98.06%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
