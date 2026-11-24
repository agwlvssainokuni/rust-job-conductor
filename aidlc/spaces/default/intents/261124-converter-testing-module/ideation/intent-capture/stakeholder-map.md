# 関係者の地図(Stakeholder Map): converter の testing モジュールの削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 3: converter の testing モジュールの削除(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 本番のコードにテストのための公開が残らないこと | 決める人(範囲、進め方、テストの行き先、比べ方の扱い) | [desc] [Q1] [Q2] [Q3] |
| `conductor` を使う人 | 変換の出力、誤りの表示、終了コードが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。移す・変える・消すテストの一覧は検索で網羅して設計の段で承認を取る | [Q1] |

## Assumptions & Open Questions

None.
