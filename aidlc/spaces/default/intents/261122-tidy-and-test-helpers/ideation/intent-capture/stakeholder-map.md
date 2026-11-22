# 関係者の地図(Stakeholder Map): 動きを変えない小さな整理とテストの補助

## Sources

- [desc] Initial description: "動きを変えない小さな整理とテストの補助(調査の 1 と 2) … 既存のテストの期待は変えない"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 重複・ずれ・到達しない分岐がなくなり、テストの補助がまとまること。動きと期待が変わらないこと | 決める人(範囲、進め方、テストの扱い、削除) | [desc] [Q1] [Q2] [Q3] [Q4] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合うこと | 影響する人 | [desc] [Q4] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。変えるテストのファイルの一覧は、検索で網羅して設計の段で承認を取る | [Q1] [Q2] |
| 足すテストの一覧を設計の段で示す。`program_name` の削除による件数の減りを記録する | [Q3] [Q4] |

## Assumptions & Open Questions

None.
