# 技術の選択: u6-attempt-times

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の FR6.6、NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C4)、team.md の Code Style。

| 項目 | 選択 | 理由 |
|---|---|---|
| 時刻の型と書式 | U5 の core `time.rs`(`Timestamp`、`format_rfc3339`、`parse_rfc3339`、`duration_secs`) | FR6.6。新しい依存が要らない |
| 現在の時刻 | U5 の cli の差し替え口 `Runtime::now()` | 流れのテストで時刻を固定できる |
| ステートの読み書き | 既存の `toml` による読み書きに `[[job.attempt]]` を足す | 今の検査の仕組み(知らないキー、型)を使える |
| 所要時間の計算と書き方 | core の純粋な関数 | `watch` のほか U7 の継続表示も使う |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR4.1 | 新しい依存を加えない | `Cargo.lock` が変わらない | FR6.6、NFR4 |
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
