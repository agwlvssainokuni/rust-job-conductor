# 関係者の地図(Stakeholder Map): 小さな直し

## Sources

- [desc] Initial description: "7 回目の調査の intent A: 小さな直し(動きを変えない。版は上げない) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: B
- [Q3] `intent-capture-questions.md` Q3: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 説明と仕様書が実装と合い、テストの重複が無く、ソースの検査が抜け道を残さないこと | 決める人(範囲、進め方、`signals_order.rs` の扱い、検査で止める書き方) | [desc] [Q1] [Q2] [Q3] |
| `conductor` を使う人 | 出力、終了コード、ステートが今までと同じこと | 影響する人 | [desc] |
| 仕様書を読んで実装する人 | 仕様書の記述が実装と合い、読み分けやすいこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。消すテストは残るテストを添えて設計の段で承認を取る | [Q1] |

## Assumptions & Open Questions

None.
