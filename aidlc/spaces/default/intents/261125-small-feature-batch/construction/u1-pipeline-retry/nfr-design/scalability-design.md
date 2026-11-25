# 規模の設計: u1-pipeline-retry

出典: 規模の要件(`scalability-requirements.md` の NFR1.4)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3)、契約(`contract-summary.md` の C3)。

単体の CLI で、水平・垂直の拡張、負荷の分散、データの分割は当てはまらない。

## 設計

- 定義のファイルの大きさの上限と、読み取りの上限は今のまま使い、このユニットでは変えない。
- 既定の複製でメモリはジョブの数に比例して増えるが、1 つは数十バイトで、今のジョブの定義(コマンド、環境変数)より小さい。上限を下げる必要は無い。
- 確かめ方は、性能の設計(`performance-design.md`)の多数のジョブのテストと共有する。既定を持つ多数のジョブの定義が、読み込みを通ることを同じテストで確かめる。
