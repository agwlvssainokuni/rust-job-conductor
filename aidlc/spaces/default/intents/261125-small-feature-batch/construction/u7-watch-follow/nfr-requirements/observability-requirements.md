# 観測の要件: u7-watch-follow

出典: 要件(`requirements.md` の FR7.4、FR8.1)、機能の仕様(`functional-spec.md` の 2、4)、ビジネスルール(`rules.md` の BR2.2、BR5.1)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.9 | 継続表示の表は、今の `watch` の表と同じ(所要時間の列を含む)。UNIX 系の端末では画面を消して描き直し、ほかは空の 1 行で区切って追記する | 表示のテスト(消す制御の文字、区切り) | FR7.4、BR2.2 |
| NFR1.10 | 使い方の表(`args.rs`、`report/messages.rs`、README)に `--follow` と `--interval` を揃えて足す | `docs` のテスト(必須の語句と使い方の一致) | FR8.1、BR5.1 |
