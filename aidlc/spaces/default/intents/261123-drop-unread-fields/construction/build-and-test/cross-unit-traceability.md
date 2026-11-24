# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 読まれない項目を型から消す | BR1.1、BR2.1、BR2.2、BR3.1、BR3.2 | `FlowEdge.explicit`、`EnvOrigin` と `origin`、`EnvFileContent.path`、`EnvEntry.line` を消した。参照は 0 件。テストは名前の並び、値、行き先で確かめる |
| SM2 | 仕様書を参照実装に合わせる | BR4.1、BR4.2 | rust と言語に依らない仕様書の S1〜S21 を直した。消す語句は 0 件、残す記述は残る |
| SM3 | 定義の `format_version` と `description` は残す | BR5.1 | 触れていない |
| SM4 | 確かめる手段 | BR9.2、BR9.3、BR9.6 | §7 の検索、テスト名の比較、`cargo doc` の警告(5 件のまま)を走らせた |
| SM5 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 期待の文字列は無変更で通る。cli・runtime・converter の src は無変更。版は 0.9.8 のまま |
| SM6 | 既存のテストの扱いと件数 | BR8.1、BR9.2、BR9.3 | テストの変更は T1〜T20、消した確かめの行は 13 行、名前の変更は ★ 2 件。1,153 − 0 + 0 = 1,153 |
| SM7 | 合格の条件 | BR9.4 | fmt、clippy、1,153 通過、行カバレッジ 98.08%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
