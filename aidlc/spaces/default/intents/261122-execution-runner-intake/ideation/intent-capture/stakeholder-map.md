# 関係者の地図(Stakeholder Map): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

## Sources

- [desc] Initial description: "残りの整理を 1 つの intent で: (1) JobExecution の列挙型化 … (3) B3: core の定義ファイルの読み込みの順序 … 段はこの順で、B3 を最後に置く"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: B

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 起動の結果が型で守られ、コマンドを分け直さないこと。2 つの読み込みの順序が揃うこと | 決める人(範囲、進め方、テストの扱い、揃える向き、版) | [desc] [Q1] [Q2] [Q3] [Q4] |
| `conductor` を使う人 | 段 1・2 では出力、終了コード、ステートが変わらないこと。段 3 では、まれな場合の誤りの種類が変わることを版と仕様書で知れること | 影響する人 | [desc] [Q3] [Q4] |
| 仕様書を読んで実装する人 | `Runtime` トレイト、`run_job`、定義ファイルの読み込みの順序の記述が、実装と合うこと | 影響する人 | [desc] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。公開の型とトレイトの変更で直すテストの一覧は、設計の段で承認を取る | [Q1] [Q2] |
| 段 3 の誤りの種類の変更を仕様書に書き、v0.9.6 に上げる | [Q3] [Q4] |

## Assumptions & Open Questions

None.
