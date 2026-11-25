# 規模の設計: u2-optional-env-file

出典: 規模の要件(`scalability-requirements.md` の NFR1.4)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3)、契約(`contract-summary.md` の C1)。

単体の CLI で、拡張や分割は当てはまらない。

## 設計

- `.env` の大きさ(1 MiB)と項目の数(10,000)の上限、定義の大きさの上限は今のまま使う。EnvFileRef はパスの文字列と真偽 1 つで、今の文字列より 1 バイト増えるだけ。
- 確かめ方は今の上限のテスト(`env_parse_perf.rs`、`env_refs_perf.rs`、`env_loader.rs` の大きさの検査)を共有する。
