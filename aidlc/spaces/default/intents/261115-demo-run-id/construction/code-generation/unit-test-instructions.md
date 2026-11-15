# Unit Test Instructions(demo-run-id)

## 実行の環境

- 言語とツール: Rust(edition 2024、最小 1.85)、`cargo`。外部のサービスやネットワークは要らない。UNIX 版のテストは、`sh` を使う。
- テストの枠組み: 標準の `cargo test`(統合は `crates/cli/tests/`)。バイナリを使うテストは、`env!("CARGO_BIN_EXE_conductor")` を使い、環境変数は `Command::env` / `env_remove` で明示する。
- 設定ファイル: 変更なし。

## この作業のテストの実行

このスコープは、Unit に分けない(zero-Unit)。変更は `conductor-cli` のテストと、`demo/` だけ。

```bash
cargo test -p conductor-cli --test unix_demo --locked       # UNIX 版のデモの検証(Step 3)
cargo test -p conductor-cli --test windows_demo --locked    # macOS・Linux では 0 件。Windows の CI で実行される(Step 5)
cargo test --workspace --locked                             # 全体の確認(Step 7)
DEMO_SLEEP=0.1 sh demo/run-demo.sh                          # デモそのものを、目で確かめる(Step 2)
```

## 整形と静的検査・最小の Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked
```

## 期待する網羅(Minimal の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| `unix_demo.rs`(UNIX) | 約 3 | デモが 0 で完走し、場面 0〜9 の見出しと要点の行が出る。利用者の環境(`CONDUCTOR_RUN_ID`・`DEMO_RUN_ID`・`LC_ALL`)に左右されない。一時ディレクトリが残らない |
| `windows_demo.rs`(Windows) | 約 6 | 場面 7(`validate -v` の 1 行と 0、誤りの定義は 64)、場面 8(`build-1` が 2 で止まり、`build-2` が別のステートで 0、`watch --run-id build-1` の `Run ID:`、同じ run ID で再開)、場面 9(`--run-id` が環境変数に勝つ、`--run-id ""` で固定、`reset --run-id`、`reset --all` のあと、ステートファイルがない)。`Demo::run` は `CONDUCTOR_RUN_ID` と `DEMO_RUN_ID` を外す |

合計: 約 10 件。既存のテストは、そのまま成功する。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。`conductor` 本体は変えないので、実質変わらない。全体の下限は、Build and Test が、全テストで判定する(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)。この測定は、ほかの命令とは別の命令として実行する。

## モックとスタブ

- モックは使わない。UNIX 版のテストは、実際の `sh` と `conductor` を使い、一時ファイルは、デモ自身の一時ディレクトリ(`mktemp -d`)に作られる。Windows 版のテストは、`tempfile` の一時ディレクトリに、定義とスクリプトを写して使う(従来どおり)。

## テストデータ

- 定義とスクリプトは、`demo/` と `demo/windows/` のものをそのまま使う(テストの中で、デモのファイルを、一時ディレクトリへ写す)。
