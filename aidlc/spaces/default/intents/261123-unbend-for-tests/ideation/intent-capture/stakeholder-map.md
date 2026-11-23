# 関係者の地図(Stakeholder Map): テストの都合で曲げられた本番のコードと、残りの整理

## Sources

- [desc] Initial description: "5 回目の調査の候補を 1 つの intent で(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: A
- [Q5] `intent-capture-questions.md` Q5: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 本番のコードがテストの都合で曲げられていないこと。説明と型が実装に合うこと | 決める人(範囲、進め方、テストの扱い、差し替え口・登録済みの印・一覧の扱い) | [desc] [Q1] [Q2] [Q3] [Q4] [Q5] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。変える・移す・消すテストの一覧は検索で網羅して設計の段で承認を取る | [Q1] [Q2] |
| パニックのフックで確かめられなくなることを記録する | [Q3] |

## Assumptions & Open Questions

None.
