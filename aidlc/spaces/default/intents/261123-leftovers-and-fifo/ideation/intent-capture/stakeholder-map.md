# 関係者の地図(Stakeholder Map): 残りの整理と読み込みの順序のそろえ

## Sources

- [desc] Initial description: "4 回目の調査の候補を 1 つの intent で(段 1 は動きを変えない整理、段 2 は利用者から見える変更、最後に版を 0.9.8 に) …"(全文は `intent-statement.md` の Sources)
- [Q1] `intent-capture-questions.md` Q1: A
- [Q2] `intent-capture-questions.md` Q2: A
- [Q3] `intent-capture-questions.md` Q3: A
- [Q4] `intent-capture-questions.md` Q4: C
- [Q5] `intent-capture-questions.md` Q5: A
- [Q6] `intent-capture-questions.md` Q6: A
- [Q7] `intent-capture-questions.md` Q7: A

## Stakeholders

| 関係者 | 関心 | 決める人か、影響する人か | Source |
|--------|------|--------------------------|--------|
| この作業を頼んだ人(開発者) | 古い説明・使われないもの・二重の処理がなくなり、読み込みの順序がそろうこと | 決める人(範囲、進め方、テストの扱い、`_job`、段 2 の範囲) | [desc] [Q1] [Q2] [Q3] [Q4] [Q5] [Q6] [Q7] |
| `conductor` を使う人 | 段 1 では動きが変わらないこと。段 2 の変更(名前付きパイプで止まらない、空白だけのコマンド名が 64)を版と仕様書で知れること | 影響する人 | [desc] [Q5] |
| 仕様書を読んで実装する人 | 読み込みの順序、コマンド名の規則、関数の説明が実装と合うこと | 影響する人 | [desc] [Q5] |

## Communication Requirements

| 必要なこと | Source |
|------------|--------|
| 結果は最後にまとめて見せる。変えるテスト・消すテストの一覧は検索で網羅して設計の段で承認を取る | [Q1] [Q2] |
| 性能のテストの直す前と後の時間を記録する | [Q6] |
| 段 2 の変更を仕様書に書き、v0.9.8 に上げる | [Q5] [desc] |

## Assumptions & Open Questions

None.
