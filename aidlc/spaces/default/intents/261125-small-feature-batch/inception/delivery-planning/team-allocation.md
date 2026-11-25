# 作る人の割り当て: v0.10.0 の小さな機能のまとまり

出典: 関係者の地図(`stakeholder-map.md`)、Bolt の計画(`bolt-plan.md`)、実装の計画の質問(`delivery-planning-questions.md`)。

Bolt は、作業を 1 周回す単位で、終わると動くものができる。チームの編成の段階は省いた(開発者ご自身 1 人の開発のため)。そのため、すべての Bolt を同じ組で作る。

| Bolt | ユニット | 作る組 |
|---|---|---|
| Bolt 1 | u1-pipeline-retry | 開発者ご自身(レビューと承認)と AI(aidlc-developer-agent。コードとテストを書く) |
| Bolt 2 | u2-optional-env-file | 同上 |
| Bolt 3 | u3-unset-env | 同上 |
| Bolt 4 | u4-graph-dot | 同上 |
| Bolt 5 | u5-log-dir | 同上 |
| Bolt 6 | u6-attempt-times | 同上 |
| Bolt 7 | u7-watch-follow | 同上 |

- 組は 1 つなので、Bolt どうしの受け渡しや、複数の組の調整は無い。
- 各 Bolt は直列に作る。前の Bolt の CI が通ってから、次の Bolt に入る(bolt-plan.md の完了の条件)。
- 開発者ご自身は Rust に不慣れなため、変更は小さな単位のコミットにし、小さな単位でレビューする(team.md の Way of Working)。
