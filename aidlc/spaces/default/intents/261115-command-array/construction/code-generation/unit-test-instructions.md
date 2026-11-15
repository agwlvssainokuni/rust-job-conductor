# Unit Test Instructions(command-array)

## 実行の環境

- 言語とツール: Rust(edition 2024、最小 1.85)、`cargo`。外部のサービスやネットワークは要らない。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は各クレートの `tests/`)。バイナリを起動するテストは、`env!("CARGO_BIN_EXE_conductor")` と、テスト専用の補助の実行ファイルを使う(`cargo test` が自動でビルドする)。
- 設定ファイル: 変更なし(ワークスペースの `Cargo.toml` と各クレートの `Cargo.toml`)。

## この作業のテストの実行

このスコープは、Unit に分けない(zero-Unit)。変更は 4 つのクレートに及ぶため、作業の確認は、次のコマンドを 1 回だけ実行する。

```bash
cargo test --workspace --locked
```

- 層ごとの確認(Step ごと)は、そのクレートだけを動かす: `cargo test -p conductor-core --locked`、`cargo test -p conductor-runtime --locked`、`cargo test -p conductor-cli --locked`、`cargo test -p conductor-converter --locked`。

## 整形と静的検査・最小の Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets
```

## 期待する網羅(Minimal の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| core: `Command` と定義の読み込み | 約 7 | `From` と `program_name`、配列が読める、型の誤り、`[]`・`[""]`・`["  ","x"]`、`["echo",""]` が有効、空の文字列は誤り、メッセージに値を含まない |
| runtime | 約 4 | 配列の引数がそのまま渡る、先頭が空の配列、存在しないプログラム、Windows の `.cmd` への配列(`cfg(windows)`) |
| cli | 約 2 | 配列のジョブの起動失敗の名前、`run` と `pipeline` で文字列と同じ結果 |
| converter | 約 2 | TOML → YAML → TOML と YAML → TOML → YAML で、同じ木 |

合計: 約 15 件。既存の 564 件は、そのまま成功する。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。全体の下限は、Build and Test が、組み立てたプロジェクトの全テストで判定する(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)。この作業のテストだけで 80% を下回っても、テストの失敗としては扱わず、結果に記録する。この測定は、ほかの命令とは別の命令として実行する。

## モックとスタブ

- 流れのテストは、既存の偽の `Runtime`・`Sink`・`Env`・`Signals` を使う。引数の受け渡しは、補助の実行ファイルで、実際の起動を確かめる(モックにしない)。

## テストデータ

- 定義の例は、テストの中の文字列(`[[job]]` の形)と、`tests/fixtures/`。`demo/` の定義は、デモの実行で確かめる。
