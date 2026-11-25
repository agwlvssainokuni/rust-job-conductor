# 技術の選択: u5-log-dir

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR5、NFR7)、機能の仕様(`functional-spec.md` の 1、4)、ビジネスルール(`rules.md` の BR2.1、BR3.4、BR4.1)、契約(`contract-summary.md` の C2、C6)、team.md の Code Style。

| 項目 | 選択 | 理由 |
|---|---|---|
| 時刻の計算 | core に新しいモジュール `time.rs`(CoreTime)。日数から年月日を求める既知の算法を自前で書く | 依存を足さない(FR6.6)。core は純粋 |
| 現在の時刻 | runtime に `clock.rs`(RuntimeClock)。`std::time::SystemTime` | `unsafe`・`cfg` が要らない(ADR-002) |
| ログのファイル | runtime に `log.rs`(RuntimeLog)。`std::fs::create_dir_all`、`OpenOptions::create_new`。0600 の指定は `platform/unix.rs`(`OpenOptionsExt::mode`)、Windows は何もしない | `cfg` を platform に閉じる(NFR5) |
| 出力の複製 | `Stdio::piped()` と、流れごとの読み取りのスレッド(`std::thread`)、ファイルは `Mutex` で順に書く | 非同期のランタイムを使わない(今の方針) |
| 差し替え口 | cli の `Runtime` に `now()` を足し、`run_job` にログの指定(無ければ `None`)を渡す | 流れのテストで時刻を決められる(ADR-002) |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
