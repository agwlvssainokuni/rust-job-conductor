# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | README のテストを 1 つに | BR1.1、BR1.2、BR1.3 | `src/tests/docs.rs` に 18 件。補助の `readme` は 1 つ。元の 10 ファイルは無い |
| SM2 | 小さなファイルをまとめる | BR2.1〜BR2.3、BR3.1〜BR3.3、BR4.1、BR4.2 | converter・cli・runtime の 9 ファイルを移した。`--remove-on-complete` の節を作り、紛れ込んだ 2 件を並べ直した |
| SM3 | 補助の写しを 1 つに | BR5.1、BR5.2、BR5.3 | `tests/common/fakes.rs` を統合テストと単体テストが読む。写しは無い |
| SM4 | ひな形の原本を 1 つに | BR6.1、BR6.2 | `tests/fixtures/init_{ja,en}.toml` を両方が読む。`.gitattributes` で LF に固定(Windows は CI で確かめる) |
| SM5 | テストの名前と中身を変えない | BR9.2、BR9.3 | 名前の差 0、件数 1,149 のまま |
| SM6 | 製品のコードの検査 | BR9.6 | 3 つの crate の検査が数と中身を変えずに通る |
| SM7 | 仕様書 | BR7.1、BR7.2、BR7.3 | rust と言語に依らない 11 章を直した。無くなったファイルの名前は 0 件 |
| SM8 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 本番のコードの差分なし。版は 0.9.8 のまま |
| SM9 | 合格の条件 | BR9.4 | fmt、clippy(本番の形と Windows を含む)、1,149 通過、行カバレッジ 98.06%。依存の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
