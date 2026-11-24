# 関係者の地図(Stakeholder Map): テストのファイルの統合

## Sources

- [desc] Initial description: "7 回目の調査の intent B: テストのファイルの統合(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: A
- [Q5] `intent-capture-questions.md` Q5: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 同じ対象のテストが 1 か所にまとまり、補助と原本の写しが 1 つになること | 決める人(範囲、進め方、README のテストの置き場所、補助と原本の扱い、run-id のテストの行き先) | [desc] [Q1] [Q2] [Q3] [Q4] [Q5] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書のテストのファイルの表が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。名前を変える・消すテストがあれば設計の段で承認を取る | [Q1] |

## Assumptions & Open Questions

None.
