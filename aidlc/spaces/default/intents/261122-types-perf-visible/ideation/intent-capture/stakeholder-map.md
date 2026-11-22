# 関係者の地図(Stakeholder Map): 型と構造の整理・計算量・利用者から見える変更

## Sources

- [desc] Initial description: "型と構造の整理・計算量・利用者から見える変更(調査の 3・4・5)を 1 つの intent で … 最後に版を 0.9.7 に上げる"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: A
- [Q5] `intent-capture-questions.md` Q5: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 型で守られ、二乗の時間が消え、誤りの表示が分かりやすくなること | 決める人(範囲、進め方、テストの扱い、表示の直し方、順序) | [desc] [Q1] [Q2] [Q3] [Q4] [Q5] |
| `conductor` を使う人 | 段 1・2 では出力・終了コード・ステートが変わらないこと。段 3 の表示の変更を版と仕様書で知れること | 影響する人 | [desc] [Q4] [Q5] |
| 仕様書を読んで実装する人 | 型、誤りの理由の文、`.env` の読み込みの順序の記述が実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。変えるテストの一覧は検索で網羅して設計の段で承認を取る | [Q1] [Q2] |
| 性能のテストの直す前と後の時間を記録する | [Q3] |
| 段 3 の表示の変更を仕様書に書き、v0.9.7 に上げる | [Q4] [Q5] [desc] |

## Assumptions & Open Questions

None.
