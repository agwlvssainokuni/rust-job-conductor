# Unit Test Instructions: 完了後もステートを残す(0.7.0)

方式は test-after。テストの量は Minimal(要件ごとに 1 件以上、各コンポーネントに 1 件以上の正常系)。既存のテストは、すべて成功のまま保つ。

## テストの環境

- フレームワーク: Rust 標準の `cargo test`(単体テストは各ファイルの `#[cfg(test)]`、統合テストは `crates/cli/tests/`)。新しい依存や設定ファイルは足さない。
- ツールチェイン: 安定版(開発は 1.97)と、最小の 1.85。

## 実行のコマンド(この作業の範囲に限る)

層ごとの実行(実装した層のあとに、その層のテストを書いて実行する):

```bash
cargo test -p conductor-core --locked
cargo test -p conductor-runtime --locked
cargo test -p conductor-cli --lib --locked
cargo test -p conductor-cli --locked --test flow_pipeline --test flow_run_id --test flow_aux --test flow_validate
cargo test -p conductor-cli --locked --test binary_basic --test binary_run_id
cargo test -p conductor-cli --locked --test unix_demo
```

`windows_demo` は `cfg(windows)` なので、開発機(macOS)では実行されない。CI の Windows で実行する。

この作業全体の確認(ビルドと検査):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo +1.85 build --workspace --locked --all-targets
```

## カバレッジ

- 下限: 行カバレッジ 80% 以上(team.md)。前回は 97.74%。測定は、開発機の `cargo llvm-cov --workspace --locked --summary-only` と、CI の `coverage`。全体のテストで測るため、Build and Test が確かめる。この作業のテストは、閾値を変えない。

## テストの方針

- 単体(core): `remove_on_complete` の読み取り(省略、`true`、`false`、型の誤り)。
- 単体(runtime): `read_state_raw` と `read_state` の差(完了のステートだけが違う)、破損・版違い・run ID の食い違いは同じ誤り。
- 単体(cli): `args.rs` のオプションの表(`--remove-on-complete` の許可と 68)、`watch` の表と完了の行(英語・日本語)。
- 統合(`flow_*`): フェイクの `Runtime` と標準出力・標準エラーの捕捉で、完了後のステートの有無(残る、`--remove-on-complete` で消える、定義の項目で消える)、再実行が先頭から始まること、`run` の判定。
- 統合(`binary_*`): 実際のバイナリで、完了後にファイルが残り、`watch` が表と完了の行を出し、`--remove-on-complete` でファイルが消えること。`conductor --version` が 0.7.0。
- デモ(`unix_demo`): 場面の出力が新しい挙動と合うこと。

## モックとテストデータ

- 既存のフェイクの `Runtime`(`crates/cli/tests/` の共通部)を使う。`read_state_raw` のフェイクは、ほかのメソッドと同じく、保持しているステートをそのまま返す。
- 実ファイルを使う統合テストは、`target/` の下の一時ディレクトリに、定義ファイルとステートを置く(既存の流儀)。
