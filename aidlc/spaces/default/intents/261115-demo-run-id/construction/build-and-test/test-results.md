# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` | 0 | 成功(`windows_demo.rs` の型検査を含む) |
| README(ルート)の必須の語句の検索(`docs-check` と同じ 7 語句) | 0 | すべてある |
| `demo/windows/run-demo.cmd` の文字コードと改行 | — | UTF-8、CRLF(230 行すべて) |
| `crates/*/src`、ルートの `README.md`、`Cargo.toml`、`Cargo.lock`、`.github/` の変更 | — | なし(`git diff 1d8a461 HEAD`) |

## テスト

命令: `cargo test --workspace --locked`(安定版)と `cargo +1.85 test --workspace --locked`。どちらも終了コード 0。合計 666 件成功、失敗 0、無視 1(converter の以前からのもの)。

| パッケージ | 成功 | 失敗 | 前回(0.5.0)との差 |
|-----------|------|------|-------------------|
| conductor-core | 133 | 0 | 0 |
| conductor-runtime | 182 | 0 | 0 |
| conductor-converter | 114 | 0 | 0 |
| conductor-cli | 237 | 0 | +3(`unix_demo.rs`) |

- 合計 663 → 666(+3)。`unix_demo.rs` の 3 件: デモが 0 で完走し場面 0〜9 の見出しと要点の行が出る、利用者の環境の run ID とロケールに左右されない、一時ディレクトリが残らない。
- `windows_demo.rs` に 6 件を足した。macOS では 0 件として動く(`cfg(windows)`)。実行は、CI の `windows-2022`。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`。終了コード 0。行 97.74%(6,828 行のうち未実行 154)、関数 97.97%、リージョン 97.56%。下限 80% を満たす(前回 97.72%)。`conductor` 本体は変えていないので、実質、変わらない。

## デモの実行(UNIX 版。release の `conductor` 0.5.0 で、実際に実行して、出力を確認)

| 実行 | 終了コード | 確認したこと |
|------|-----------|--------------|
| `DEMO_SLEEP=0.1 sh demo/run-demo.sh`(日本語) | 0 | 場面 0〜9 が完走 |
| `DEMO_LANG=en_US.UTF-8 ...`(英語) | 0 | 英語の表示 |
| `CONDUCTOR_RUN_ID=zzz DEMO_RUN_ID=zzz LC_ALL=C ...` | 0 | `zzz` は使われず、既存の場面が run ID なしで動き、表示は日本語のまま |

新しい場面の確認: 場面 7(`validate -v` が 0 と `定義は正しいです(ジョブ 4 個)`、command の引用符が閉じていない定義が 64 と `command の引用符が閉じていません`、ステートの一覧が `(なし)`)、場面 8(`build-1` が 2 で止まり、`build-2` が 0 で完走、一覧は `build-1` だけ、`watch --run-id build-1` に `Run ID: build-1`、同じ run ID で再開すると `extract`・`transform` はスキップ、完了で一覧が空)、場面 9(環境変数だけで `env-1` が 2、`--run-id opt-1` が環境変数に勝つ、`--run-id ""` が `conductor.toml.state` に固定、`opt-2`、一覧 4 個、`reset --run-id opt-1` で 3 個、`reset --all` が 0 で空、`watch` が `no state`)。

## 実行できていないもの

- **Windows 版 `demo/windows/run-demo.cmd` の実行:** macOS では実行できず、CI でも実行されない。既存の場面の書き方に合わせて読み直しただけ。実機での確認が要る点: `echo` の並びによる `bad.toml` の作成、`--run-id ""` が空の引数として渡ること、`dir /b` が一致なしで `errorlevel 1` を返すことへの依存。
- **`windows_demo.rs` の実行:** 型検査まで。実行は、push 後の CI の Windows。
