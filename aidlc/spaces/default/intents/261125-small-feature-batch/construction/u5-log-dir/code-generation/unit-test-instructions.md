# 単体テストの手順: u5-log-dir

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、runtime は `crates/runtime/tests/`(実際のプロセスとファイル、`tempfile`)、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`)、実バイナリのテストは `crates/cli/tests/binary_*.rs`・`signals_real.rs`・`unix_demo.rs`、cli の内部のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。新しいテストのファイルは、作る前は `--test` に書けないので、Step 1 では今あるものだけを走らせる。

```bash
cargo test -p conductor-core --test time_format --test definition_valid --test definition_errors --test nfr_security --locked
```

```bash
cargo test -p conductor-runtime --test log_file --test runner_log --test runner_basic --locked
```

```bash
cargo test -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test binary_log --test signals_real --test unix_demo --locked
```

- Step 1(変更の前)は、まだ無い `time_format`・`log_file`・`runner_log`・`binary_log` を外して走らせる。
- 型の変更は多くのテストの組み立てに及ぶため、最後に `cargo test --workspace --locked` で全体が通ることも確かめる。
- `windows_demo.rs` と Windows のパイプの動きは、CI の windows の `test` で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。ユニットの確認のコマンド(`cargo llvm-cov --workspace ... --fail-under-lines 80`)と Build and Test が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。
- 新しいモジュール(`time.rs`、`clock.rs`、`log.rs`)と `runner.rs` の新しい分岐は、規則ごとに正しい場合と誤り・境界の場合のテストで通る。

## 偽物と差し替え

- core のテストは差し替え無し(純粋)。
- runtime のテストは、一時ディレクトリと、テスト用の補助バイナリ(リリースに含めない今のもの)で実際に動かす。書き込みの失敗だけは、`runner.rs` の `mod tests` で失敗する `Write` を内部の関数に渡して確かめる。
- cli の流れのテストは、偽の `Runtime` の `now()` と `run_job` の戻り値で、時刻とログの結果を決める。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- 定義は各テストの中の文字列で作る。ログのディレクトリは一時ディレクトリの中に置く。
- 時刻の表は、境界(うるう年、月末、範囲の端)を並べた表にする。
