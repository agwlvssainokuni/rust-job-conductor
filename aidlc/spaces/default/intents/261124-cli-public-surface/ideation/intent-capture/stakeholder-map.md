# 関係者の地図(Stakeholder Map): cli のライブラリの公開範囲を狭める

## Sources

- [desc] Initial description: "6 回目の調査の intent 4: cli のライブラリの公開範囲を狭める(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | cli のライブラリにテストのための公開が残らないこと | 決める人(範囲、進め方、テストの行き先、公開に残すもの) | [desc] [Q1] [Q2] [Q3] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。移す・変える・消すテストの一覧は検索で網羅して設計の段で承認を取る | [Q2] |

## Assumptions & Open Questions

None.
