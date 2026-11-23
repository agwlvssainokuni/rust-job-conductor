# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1: 型と構造 | BR1.1〜BR1.11、BR9.1 | コミット 1〜21。計画・図・リトライの型を列挙型に、読まれない項目を削除、cli の準備の入口と表示の関数の形を統一、`pub` を狭めた。期待の文字列は無変更で通る。N2 で日本語の文が変更前と同じことを確かめた |
| SM2 | 段 2: 計算量 | BR2.1〜BR2.6、BR9.1 | コミット 22〜30。性能のテスト 5 件(N4〜N8)が上限の中。直す前と後の時間を記録した |
| SM3 | 段 3: 利用者から見える変更 | BR3.1、BR3.2、BR8.1 | コミット 31〜36。文法の誤りの理由が形式ごとの文(N10)。`.env` は形 B(N11 は直す前に落ちた、N12 は前後とも通った)。仕様書に「v0.9.6 までは…」 |
| SM4 | 版 | BR3.3 | コミット 37。0.9.7(`ee9f409` と同じ 22 ファイル)。タグとリリースはしていない |
| SM5 | 既存のテストの扱いと件数 | BR8.1、BR9.2、BR9.3 | テストの変更は T1〜T26。テスト名の差は追加の 11 件だけ。1,146 + 11 = 1,157 |
| SM6 | 合格の条件 | BR9.4 | fmt、clippy、1,157 通過、行カバレッジ 98.10%。`Cargo.lock` の差分は版の行だけ。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
