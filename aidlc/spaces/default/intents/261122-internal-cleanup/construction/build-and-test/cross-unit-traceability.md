# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。成功の指標(`ideation/intent-capture/intent-statement.md`)と、確かめた結果の対応。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1(cli の整理) | BR1.1〜BR1.4 | コミット 8c4b8de〜1e4e6cf。件数 1,123 のまま。確認担当が検査の順序を照合 |
| SM2 | 段 2(型の整理) | BR2.1〜BR2.4 | コミット 6cb07b7〜ce74418。件数 1,123 のまま。テストの変更は承認済みの `report_messages.rs` だけ |
| SM3 | 段 3(core の整理) | BR3.1〜BR3.3 | コミット c34d3f7〜8e25bbf。件数 1,123 のまま |
| SM4 | 段 4(テストと文書) | BR4.1〜BR4.5 | graph 64 → 58(まとめた 6 件を記録)、converter 117 件を改名、`fake_wait`、README、凍結の明記 |
| SM5 | 動きを変えない | BR5.1、BR5.2 | 出力の期待の文字列に差分なし。段 1〜3 でテストのファイルを変えていない(例外 1 か所) |
| SM6 | 合格の条件 | BR5.3 | fmt、clippy、1,117 通過、行カバレッジ 98.19%。依存・`unsafe`・`cfg` の追加なし。MSRV は CI |

見送った軽い指摘(src のテストの補助の重複、到達しない分岐)は `construction/code-generation/code-summary.md` に記録した。
