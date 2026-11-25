# 規模の設計: u7-watch-follow

出典: 規模の要件(`scalability-requirements.md` の NFR1.8)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C5)。

## 設計

- 繰り返しの間に持ち越すのは `seen`、`absent_once`、`read_failures` の 3 つの小さな値だけ(`reliability-design.md` の 3)。ステートと表は毎回作って捨てる。追記の出力も書いたら残さない。
- 間隔は 1〜3600 秒の整数(`u32`)。
- 確かめ方: 流れのテストで 1,000 回繰り返しても終わり方が変わらないこと(偽の待ちで時間を使わない)。
