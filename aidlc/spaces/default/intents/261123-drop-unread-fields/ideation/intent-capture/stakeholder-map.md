# 関係者の地図(Stakeholder Map): 読まれない項目の削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 2: 読まれない項目の削除(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: B

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 型に読まれない項目が残らないこと | 決める人(範囲、進め方、テストの扱い、仕様書の扱い、定義の項目の扱い) | [desc] [Q1] [Q2] [Q3] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで別の言語で作り直す人 | 言語に依らない仕様書のデータの形が、参照実装と合うこと | 影響する人 | [desc] [Q2] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。変える・消すテストの一覧は検索で網羅して設計の段で承認を取る | [Q1] |

## Assumptions & Open Questions

None.
