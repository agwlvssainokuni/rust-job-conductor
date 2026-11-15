# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功。`conductor --version` は `conductor 0.5.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` | 0 | 成功 |
| README の必須の語句の検索(`docs-check` と同じ 7 語句) | 0 | すべてある |

## テスト

命令: `cargo test --workspace --locked`(安定版)と `cargo +1.85 test --workspace --locked`。どちらも終了コード 0。合計 663 件成功、失敗 0、無視 1(converter の以前からのもの)。

| パッケージ | 成功 | 失敗 | 前回(0.4.0)との差 |
|-----------|------|------|-------------------|
| conductor-core | 133 | 0 | +11 |
| conductor-runtime | 182 | 0 | +20 |
| conductor-converter | 114 | 0 | 0 |
| conductor-cli | 234 | 0 | +35 |

- 合計 597 → 663(+66)。内訳は、core 11(`run_id.rs` の境界 8、定義の `run_id_env` 3)、runtime 20(単体 5、`run_id_state.rs` 15)、cli 35(`args_grammar.rs` 7、`flow_run_id.rs` 23、`binary_run_id.rs` 5)。
- 既存のテストの修正は、署名・フィールドの追従が中心。意味を変えたのは `flow_run.rs` と `flow_pipeline.rs` の 2 か所(「ロケールを判定しない」の確認を、環境変数の読み取り回数 0 から、ロケールの変数を読んでいないことに変えた。`CONDUCTOR_RUN_ID` を読むのは仕様)。
- UNIX 専用のテスト: `reset --all` がシンボリックリンクを辿らないテスト。Windows では、実行されない(CI の Windows は、このテストを持たない)。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`。終了コード 0。行 97.72%(6,828 行のうち未実行 156)、関数 97.97%、リージョン 97.52%。下限 80% を満たす(前回 97.79%)。新しいファイル: `core/src/run_id.rs` 99.07%、`runtime/src/state/reset_all.rs` 89.00%。

## 実際のバイナリでの確認(Code Generation の中で実施。release ビルド)

- 異なる run ID(`r1`、`r2`)の `pipeline` を並行して動かすと、`.state` が別々にできる。`r1` を同じ run ID で再開して完了すると、`r1` のステートだけが消え、`r2` は残る。
- `--run-id ""` は、環境変数があっても run ID なしに固定。オプションは環境変数に優先。
- 定義の `run_id_env` は、`run`・`watch`・`reset` のすべてで使われる。壊れた定義でも、`reset` は既定の `CONDUCTOR_RUN_ID` で動く。
- `reset --all` は、run ID なし・`opt`・`r1`・`r2` のステートを消し、別の定義のステート、壊れたもの、一時ファイルを残す。壊れたものは警告に出て、終了コードは 0。
- `a/b`、`..`、65 文字は 68。環境変数の値の違反も 68。`validate --run-id x` と `reset --all --run-id x` は 68。不正な `run_id_env` は 64。

## デモ

`DEMO_SLEEP=0.1 sh demo/run-demo.sh`: 終了コード 0(0.5.0 の `conductor` で実行)。
