# テストの結果: v0.10.0

出典: ビルドの手順(`build-instructions.md`)、結合テストの手順(`integration-test-instructions.md`)、各ユニットの単体テストの手順(`construction/*/code-generation/unit-test-instructions.md`)。実行は 2026-11-26、macOS(開発機)、HEAD は 7 ユニットと「版を 0.10.0 にする」(e112426)の後。

## ビルド

| コマンド | 終了コード | 結果 |
|---|---|---|
| `cargo build --workspace --locked` | 0 | 成功 |
| `cargo build --release --locked -p conductor-cli` | 0 | 成功 |
| `./target/release/conductor --version` | 0 | `conductor 0.10.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告なし |

## 各ユニットの単体テスト(重複を除いて 1 回ずつ)

| コマンド | ユニット | 終了コード |
|---|---|---|
| `cargo test -p conductor-core --test definition_retry --test definition_valid --test nfr_security --test definition_perf --locked` | u1 | 0 |
| `cargo test -p conductor-cli --test flow_retry --test flow_validate --test binary_basic --test unix_demo --locked` | u1 | 0 |
| `cargo test -p conductor-cli --lib --locked -- tests::graph_report tests::plan_report tests::docs` | u1 | 0(下の「直したこと」を参照) |
| `cargo test -p conductor-core --test env_definition --test env_merge --test env_types --test nfr_security --test env_refs_perf --locked` | u2 | 0 |
| `cargo test -p conductor-runtime --test env_loader --test flow_with_state --locked` | u2 | 0 |
| `cargo test -p conductor-cli --test flow_env --test flow_dry_run --test binary_env --test unix_demo --locked` | u2 | 0 |
| `cargo test -p conductor-core --test env_definition --test env_file_parse --test env_merge --test env_merge_perf --test nfr_security --locked` | u3 | 0 |
| `cargo test -p conductor-runtime --test runner_basic --test env_loader --locked` | u3 | 0 |
| `cargo test -p conductor-cli --test flow_env --test flow_validate --test flow_graph --test flow_dry_run --test binary_env --test unix_demo --locked` | u3 | 0 |
| `cargo test -p conductor-cli --test flow_graph --test binary_graph --test unix_demo --locked` | u4 | 0 |
| `cargo test -p conductor-cli --lib --locked -- tests::graph_dot tests::graph_report tests::docs` | u4 | 0 |
| `cargo test -p conductor-core --test time_format --test definition_valid --test definition_errors --test nfr_security --locked` | u5 | 0 |
| `cargo test -p conductor-runtime --test log_file --test runner_log --test runner_basic --locked` | u5 | 0 |
| `cargo test -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test binary_log --test signals_real --test unix_demo --locked` | u5 | 0 |
| `cargo test -p conductor-core --test attempt_flow --test time_format --test flow_sequences --test retry_flow --locked` | u6 | 0 |
| `cargo test -p conductor-runtime --test state_read --test state_write --test nfr_content --test flow_with_state --test run_id_state --locked` | u6 | 0 |
| `cargo test -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test flow_aux --test flow_graph --test flow_dry_run --test binary_basic --test unix_demo --locked` | u6 | 0 |
| `cargo test -p conductor-core --test pipeline_activity --test attempt_flow --locked` | u7 | 0 |
| `cargo test -p conductor-cli --test flow_aux --test flow_retry --test flow_run --test flow_pipeline --test binary_basic --test unix_demo --locked` | u7 | 0 |
| `cargo test -p conductor-cli --lib --locked -- tests::args_grammar tests::docs tests::report_messages` | u7 | 0 |

`docs` だけのコマンド(u2・u5・u6 の手順の補足)は、上の `tests::docs` を含むコマンドで走っている。

## 全体のテスト(結合テストを含む)

| コマンド | 終了コード | 結果 |
|---|---|---|
| `cargo test --workspace --locked`(3 回) | 0、0、0 | 毎回 1,429 件通過、失敗 0、無視 1(前からのもの)、テストのバイナリ 89 |

u6 の作業中に 1 度だけ報告された原因不明の失敗は、この 3 回と、カバレッジの実行を含めて再び起きなかった。

## カバレッジ

| コマンド | 終了コード | 結果 |
|---|---|---|
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | 0 | 行 98.18%(10,485 行中 191 行が未実行)、領域 97.86%、関数 98.15% |

## 依存

- `Cargo.lock` の package の数は v0.9.9(1f88e59)と同じ 46。変わったのは自分の 4 つのクレートの版だけ。

## 直したこと(この段階の中の直し、1 回目)

- u1 の単体テストの手順の 3 つ目のコマンドに `--` が無く、`cargo test` が 2 つ目の絞り込みを引数の誤りにした(終了コード 1、`error: unexpected argument 'tests::plan_report' found`)。テストの失敗ではなく手順の書き方の誤り。`u1-pipeline-retry/code-generation/unit-test-instructions.md` に `--` を足し、走らせ直して 63 件通過、失敗 0。

## ここでは走らせていないもの

- Windows だけのテスト(`windows_demo.rs`、`unset_env` の大文字小文字を区別しない消し方、`watch --follow` の追記の表示、ログの Windows の扱い)と、ubuntu・最小の版(1.100.0)での実行。CI の `test`(3 つの OS)と `msrv-test` が走らせる。後の CI Pipeline の段階が持つ。
- 起動の性能(`startup-bench`)は CI で警告のみ(team.md)。
