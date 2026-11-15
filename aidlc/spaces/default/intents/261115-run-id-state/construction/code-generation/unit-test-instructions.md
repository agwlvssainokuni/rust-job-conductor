# Unit Test Instructions(run-id-state)

## 実行の環境

- 言語とツール: Rust(edition 2024、最小 1.85)、`cargo`。外部のサービスやネットワークは要らない。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は各クレートの `tests/`)。バイナリを起動するテストは、`env!("CARGO_BIN_EXE_conductor")` を使い、環境変数は `Command::env` / `env_remove` で制御する。
- 設定ファイル: 変更なし。

## この作業のテストの実行

このスコープは、Unit に分けない(zero-Unit)。層ごとの確認は、そのクレートだけを動かし、最後に全体を 1 回動かす。

```bash
cargo test -p conductor-core --locked       # Step 2〜3
cargo test -p conductor-runtime --locked    # Step 4〜5
cargo test -p conductor-cli --locked        # Step 6〜7
cargo test --workspace --locked             # 全体の確認(Step 10)
```

## 整形と静的検査・最小の Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked
```

## 期待する網羅(Minimal の方針 + NFR3 の境界のテスト)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| core: `RunId` と環境変数名 | 約 6 | 境界の入力(有効: `a`・`run-1.2_x`・64 文字・`CON`。無効: 空・65 文字・`.a`・`..`・`a/b`・`a b`・`é`・制御文字)、`is_valid_env_name`、`peek_run_id_env`(壊れた入力・巨大な入力でパニックしない) |
| core: 定義の `run_id_env` | 約 2 | 正しい値、型の誤り・空・不正な名前が 64 |
| runtime: ステート | 約 7 | パス、読み書きの往復、run ID なしの形が不変、照合(一致・不一致・欠落・完了のステートでも検査)、run ID ごとの独立、`reset_state` |
| runtime: `reset_all_states` | 約 4 | 対象の判定(消すもの・消さないもの)、壊れたもの、一時ファイルと無効な名前、シンボリックリンク |
| cli: 引数と解決 | 約 5 | 4 つのコマンド、許可されない場所、併用・重複・無効な値、`resolve_run_id`(優先順位、空、固定) |
| cli: 流れ(偽の `Runtime`) | 約 4 | run ID ごとのステート、`run_id_env` の名前、`watch`・`reset` の定義の読み方(壊れた定義でも `reset`)、`Run ID:` の行、`reset --all` の警告と終了コード |
| cli: `validate` | 約 1 | `run_id_env` の誤りが 64、`--run-id` は 68 |
| cli: 実際のバイナリ | 約 3 | 2 つの run ID の並行実行と再開、`reset --all`、`.state` の名前 |

合計: 約 30 件。署名・フィールドの追加に伴う既存のテストの修正は、追従だけ。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。全体の下限は、Build and Test が、全テストで判定する(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)。この作業のテストだけで 80% を下回っても、テストの失敗としては扱わず、結果に記録する。この測定は、ほかの命令とは別の命令として実行する。

## モックとスタブ

- 流れのテストは、既存の偽の `Runtime`・`Sink`・`Env`・`Signals` を使う(偽の `Runtime` には、新しいメソッドも足す)。`runtime` のテストと、バイナリのテストは、`tempfile` の一時ディレクトリの実際のファイルを使う(モックにしない)。

## テストデータ

- 定義とステートの例は、テストの中の文字列(`[[job]]` の形)と、一時ディレクトリのファイル。
