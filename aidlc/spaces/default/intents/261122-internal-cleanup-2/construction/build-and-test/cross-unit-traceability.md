# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1: 説明と制約 | BR1.1〜BR1.8 | コミット 9 件(b3a2973〜6aaac31)。core と converter に `forbid(unsafe_code)`、上限の数を const assert で結んだ。件数は 1,128 のまま |
| SM2 | 段 2: 重複と計算量 | BR2.1〜BR2.11 | コミット 16 件(c5442e1〜3f0a0c6)。性能のテスト 3 件が上限の中(A2 5.3 ms、A3a 5.1 ms、A3b 8.9 ms)。`.env` の解析の重複の除去(任意)は見送り |
| SM3 | 段 3: cli の ctx | BR3.1〜BR3.4 | コミット 5 件(6a3a252〜5953860)。既存の流れのテストは無変更で通った |
| SM4 | 段 4: 型とテスト | BR4.1〜BR4.4 | コミット 6 件(53c533a〜80c66d9)。T2〜T7、T9。`flow_with_state.rs` は 20 件 |
| SM5 | 動きを変えない | BR5.1〜BR5.3 | 期待する出力の文字列の差分なし。テストのファイルの差分は T1〜T9 と新しい 4 ファイルだけ |
| SM6 | 合格の条件 | BR5.4 | fmt、clippy、1,137 通過、行カバレッジ 98.28%。依存の追加なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
