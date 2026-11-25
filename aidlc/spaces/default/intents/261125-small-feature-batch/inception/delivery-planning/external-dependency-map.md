# 外部への依存: v0.10.0 の小さな機能のまとまり

出典: 意図の表明(`intent-statement.md`)、実装の計画の質問(`delivery-planning-questions.md`)、チームの実践(`team-practices.md`)。

Bolt は、作業を 1 周回す単位で、終わると動くものができる。

この作業は、開発者ご自身と AI だけで進め、外部の API、データ、他のチームの受け渡し、承認の待ちは無い。新しい外部ライブラリも足さない(project.md の Forbidden)。

| 外部の項目 | 持ち主 | 待ち時間 | 止める Bolt | 遅れたときの手当て |
|---|---|---|---|---|
| GitHub Actions の CI(3 つの OS、MSRV、カバレッジ) | GitHub | 1 回の push ごとに数分 | すべての Bolt(完了の条件) | CI が動かないときは、ローカルの macOS で確かめたうえで待つ。他の OS の違いは、CI が戻ってから直す |
| Graphviz(`dot` コマンド) | 利用者の環境 | 無し | Bolt 4 の見せるものだけ | テストは DOT の文字列を比べ、Graphviz を要らないようにする |

そのほかに、外部の待ちは無い。
