# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 本体の整理 | BR1.1〜BR6.4 | 段 1〜6 の 35 コミット。`functional-spec.md` §7 の検索で、古い名前・到達しない分岐・文字列での比較は 0 件、まとめた定義は 1 か所ずつ。`program_name` は無い |
| SM2 | テストの補助 | BR7.1〜BR7.10 | 段 7 の 13 コミット。補助は共通の場所に 1 か所ずつ。`JobExecution` の手組みは 0 件。cli の偽物は `writes` を持たず、`run_lines` を記録する |
| SM3 | 新しいテスト | BR8.1 | N1〜N5 の 5 件 |
| SM4 | 利用者から見える動きは変わらない | BR9.1 | 期待する出力の文字列は無変更で通る。確認担当が文言の定数の統合で文字列が変わらないことを確かめた。版は 0.9.6 のまま |
| SM5 | 既存のテストの扱いと件数 | BR9.2、BR9.3 | テストの変更は T1〜T46。テスト名の差は T38 の削除・N1〜N5・移動の 8 件だけ。1,142 − 1 + 5 = 1,146 |
| SM6 | 合格の条件 | BR9.4 | fmt、clippy、1,146 通過、行カバレッジ 98.14%。依存の追加なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
