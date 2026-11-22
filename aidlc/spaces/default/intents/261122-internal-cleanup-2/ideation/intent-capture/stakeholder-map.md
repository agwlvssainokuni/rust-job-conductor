# 関係者の地図(Stakeholder Map): 動きを変えない整理の第 2 弾

## Sources

- [desc] Initial description: "動きを変えない整理の第 2 弾(調査の候補のうち B3・F・JobExecution の列挙型化を除く全部)… 利用者から見える動きは変えない"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 説明のずれと重複がなくなり、型で守られること。動きが変わらないこと | 決める人(範囲、進め方、テストの扱い) | [desc] [Q1] [Q2] [Q3] [Q4] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと。大きな `.env` でも遅くならないこと | 影響する人 | [desc] [Q4] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。公開の型の変更で直すテストの一覧は、設計の段で承認を取る | [Q1] [Q2] |
| 性能のテストの、直す前と後の時間を記録する | [Q4] |

## Assumptions & Open Questions

None.
