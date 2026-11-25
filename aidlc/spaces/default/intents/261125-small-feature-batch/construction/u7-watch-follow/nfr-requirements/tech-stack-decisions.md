# 技術の選択: u7-watch-follow

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR2.2)、契約(`contract-summary.md` の C5)、team.md の Code Style。

| 項目 | 選択 | 理由 |
|---|---|---|
| 間隔の待ちと中断 | 今の `Runtime::wait(secs, signal)` と合図の準備(`run`・`pipeline` と同じ) | 待ちの途中の合図で打ち切れる。偽物でテストできる |
| 画面を消す | ANSI の制御の文字(画面を消し、カーソルを左上に)を自前で書く。UNIX 系かどうかは `platform` で決める | 新しい依存が要らない(team.md の Code Style) |
| 終わりの判定 | core の純粋な関数 `pipeline_activity` | テストしやすく、表示と切り離せる |
| 引数 | 今の `args.rs` の文法に `--follow`、`--interval` を足す | 使い方の表と誤りの表示を共有できる |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR4.1 | 新しい依存を加えない | `Cargo.lock` が変わらない(版の更新による自分のクレートの版の行を除く) | NFR4 |
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
