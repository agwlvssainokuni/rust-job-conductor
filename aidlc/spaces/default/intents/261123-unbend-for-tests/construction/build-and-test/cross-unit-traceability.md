# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1: テストの都合で曲げられた所 | BR1.1〜BR1.10 | 本番から、テストのためだけの関数・項目・引数・公開範囲・差し替え口を消した。テストは公開の入口、偽物の記録、テストの補助(`tests/common/kinds.rs`)、単体テストで代わりを持つ。パニックのフックで確かめられなくなることを仕様書に記録した。消した名前の参照は 0 件 |
| SM2 | 段 2: ふつうの整理 | BR2.1〜BR2.6 | 古い説明、`definition_key`・`job_order`、`CommandErrorKind::reason()`、`ExitCodeClass`、小さい整理、`decide_from_state` を直した。ステートファイルのバイト列は前後で同じ |
| SM3 | 確かめる手段 | BR9.2、BR9.3 | `inventory.md` と functional-spec §7 の検索を走らせ、期待どおりだった |
| SM4 | 動きを変えない、版を上げない | BR9.1、BR9.5 | 期待の文字列は無変更で通る。版は 0.9.8 のまま |
| SM5 | 既存のテストの扱いと件数 | BR8.1、BR9.2、BR9.3 | テストの変更は T1〜T31、消したテストは §4.6 の 5 件、足したテストは N1。1,161 − 5 + 1 = 1,157 |
| SM6 | 合格の条件 | BR9.4 | fmt、clippy、1,157 通過、行カバレッジ 98.12%。`Cargo.toml`・`Cargo.lock` の差分なし。MSRV と他の OS は CI で通った |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
