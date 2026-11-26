# 単体テストの手順: v0.10.1 の整理・修正

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。runner はすでに動く(brownfield)。
- runtime の単体テストは `crates/runtime/src/runner.rs` の `mod tests`、統合テストは `crates/runtime/tests/`。core の流れの単体テストは `crates/core/src/flow/` の `mod tests`、計画・流れ・性能のテストは `crates/core/tests/`。cli の実バイナリのデモのテストは `crates/cli/tests/unix_demo.rs`、文書のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## この作業のテストの走らせ方

この作業が変える所のテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

F2(runtime):

```bash
cargo test -p conductor-runtime --test log_file --test runner_log --locked
```

```bash
cargo test -p conductor-runtime --lib --locked -- runner::tests
```

F3(core。Step 6 で直すときに、流れと計画と性能のテストが変えずに通ることを確かめる):

```bash
cargo test -p conductor-core --lib --locked -- flow::
```

```bash
cargo test -p conductor-core --test plan_pipeline --test plan_run --test plan_assume --test plan_simulate_perf --test flow_sequences --test flow_perf --test retry_flow --test attempt_flow --locked
```

F5・F1・文書のずれ(cli と core):

```bash
cargo test -p conductor-cli --test unix_demo --locked
```

```bash
cargo test -p conductor-cli --lib --locked -- tests::docs tests::report_messages
```

```bash
cargo test -p conductor-core --test env_types --locked
```

- 変更が runtime・core・cli の広い範囲に及ぶため(版の更新と F3 の直し)、Step 4・10・11 で `cargo test --workspace --locked` で全体が通ることも確かめる。
- `unix_demo.rs` は UNIX 系でだけ走る。`windows_demo.rs` と Windows の動きは CI の windows で確かめる。
- F3 の測定(Step 5・6)は使い捨てのテストで行い、上のコマンドには含めない。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。Build and Test が全体で測る(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`)。上のコマンドでは判定しない。下限は下げない。
- `plan_simulate_perf.rs` の上限 150ms は変えない。

## 偽物と差し替え

- `copy_stream` のテストは、`runner.rs` の `mod tests` の中の今の補助(`Shared::new` など)と、`read` の回数を数える読み手で書く。本番にテスト専用の引数・`pub`・関数を足さない。
- F3 の判断の一致のテスト(R-06)は、`flow` の `mod tests` の中で、内部の入口と公開の入口を同じ入力で呼んで比べる。
- デモのテストは、今の `run_demo` に `DEMO_PAUSE=1` を足すだけで、標準入力は `run_limited` の既定(空)のまま。

## テストのデータ

- 定義とステートは各テストの中の文字列か、今の補助(`crate::test_support` など)で作る。ログの名前の表は `log_file.rs` の今の表に行を足す。
