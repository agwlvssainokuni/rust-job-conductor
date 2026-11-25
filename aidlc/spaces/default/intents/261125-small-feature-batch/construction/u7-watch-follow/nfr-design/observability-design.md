# 観測の設計: u7-watch-follow

出典: 観測の要件(`observability-requirements.md` の NFR1.9、NFR1.10)、信頼性の要件(`reliability-requirements.md` の NFR1.5)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、4)、契約(`contract-summary.md` の C5)。

## 設計

- **描き方(NFR1.9)**: `DisplayEvent::WatchTable` と `WatchEmpty` の文字列を作る関数はそのまま使い、継続表示の流れが前に置くもの(画面を消す文字列、または 2 回目からの空の 1 行)だけを足す。表の中身は 1 回の `watch` と同じ。
- **使い方の表(NFR1.10)**: `args.rs` の `watch` の項に `--follow`、`--interval <秒>` を足し、`report/messages.rs` の使い方の文(英日)と README の使い方を揃える。`src/tests/docs.rs` の必須の語句に `--follow` と `--interval` を足す。
- **README の記載(NFR1.5。非機能の要件のレビューの指摘)**: 継続表示の終わる条件、終了コード、Windows・パイプでの追記、エラーのジョブの後に続くジョブがある定義では間の短い時間に止まったと判定して終わることがある点を書く。終わる条件の節の見出しの語(「止まった」など)を `docs.rs` の必須の語句に足し、記載が消えないようにする。
- 確かめ方: 表示のテスト、`docs` のテスト。
