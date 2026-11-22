# テスト結果(Test Results): v0.9.0 の機能拡張

実行した日: 2026-11-21〜2026-11-22。環境: macOS(Darwin)、`rustc 1.100.0`、edition 2024。リポジトリの状態: `main` の、コミット `705a979` 以降(ユニット U4 のコード生成のあと)。

## ビルド

- `cargo build --workspace --locked`: 成功。
- `cargo fmt --all -- --check`: 通った(差分なし)。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。

## テスト(プロジェクト全体)

- `cargo test --locked --workspace`: **合計 1,017 件が通り、失敗 0 件**(91 のテストの塊。skipped の記録なし)。

## ユニットごとの、範囲を絞ったテスト(各ユニットの `unit-test-instructions.md` の、重複を除いた 15 のコマンドを、1 回ずつ実行)

| # | コマンドの対象 | 通った | 失敗 | 終了コード |
|---|----------------|--------|------|------------|
| 1 | cli: `args_grammar`、`report_messages`、`escape_table`(+ 流れの既存テスト) | 88 | 0 | 0 |
| 2 | cli: 同上(別の組み合わせ) | 198 | 0 | 0 |
| 3 | cli: `flow_env`、`binary_env`、`docs_env`(U1) | 17 | 0 | 0 |
| 4 | cli: `graph_report`、`flow_graph`、`binary_graph`、`docs_graph`(U2) | 22 | 0 | 0 |
| 5 | cli: `init_types`、`init_template`、`flow_init`、`binary_init`、`docs_init`(U4) | 26 | 0 | 0 |
| 6 | cli: `plan_report`、`flow_dry_run`、`binary_dry_run`、`docs_dry_run`(U3) | 23 | 0 | 0 |
| 7 | cli: `report_messages`、`escape_table` | 39 | 0 | 0 |
| 8 | cli: `report_messages`、`escape_table`、`args_grammar`、`flow_aux` ほか | 121 | 0 | 0 |
| 9 | core: `--lib` | 88 | 0 | 0 |
| 10 | core: `--lib`、`flow_sequences`、`retry_flow` | 113 | 0 | 0 |
| 11 | core: `env_types`、`env_definition`、`env_file_parse`、`env_merge`(U1) | 32 | 0 | 0 |
| 12 | core: `graph_types`、`graph_edges`、`graph_overlay`、`graph_drawable`、`graph_layout_oracle`、`graph_perf`(U2) | 31 | 0 | 0 |
| 13 | core: `plan_types`、`plan_pipeline`、`plan_assume`、`plan_run`、`plan_perf`(U3) | 23 | 0 | 0 |
| 14 | runtime: `env_loader`、`runner_env`(U1) | 12 | 0 | 0 |
| 15 | runtime: `file_create`(U4) | 8 | 0 | 0 |

(ユニットのコマンドは、互いに重なる範囲を含むので、通った件数の合計は、全体の 1,017 件とは一致しない。件数の二重の数えを、避けるため、合計は、全体の実行の値を使う。)

## カバレッジ(プロジェクト全体。1 回の、全体の実行)

- コマンド: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`
- 終了コード: 0。
- 行カバレッジ: **97.94%**(9,308 行のうち未通過 192 行)。領域 97.60%、関数 97.98%。下限(80%)を超えている。
- 新しいコードの主なファイル: `core/src/envfile/merge.rs` 100%、`core/src/graph/edges.rs` と `overlay.rs` 100%、`cli/src/report/graph_text.rs` 99.4%、`core/src/plan/simulate.rs` 98.5%、`cli/src/report/init_template.rs` 100%、`cli/src/report/plan_text.rs` 100%、`runtime/src/file_create.rs` 88.0%、`cli/src/dry_run.rs` 86.3%、`cli/src/graph.rs` 94.2%、`runtime/src/envfile.rs` 93.9%。

## 失敗の詳細

なし(失敗したテストは 0 件)。ただし、次の「揺れ」を記録する(失敗ではない)。計測つきの実行では、マシンが忙しいとき、時間のかかる 2 つのテスト(`unix_demo` の一部、`conductor-converter` の `expanded_size_budget_boundary_with_the_default_16_mib`)が、30 秒の上限を超えて失敗することが、開発の途中で見られた。単独で再実行すると通る。今回の、段階の中の実行(上の表)では、起きなかった。

## 実行しなかったもの(理由つき)

- `crates/cli/tests/windows_demo.rs`(場面 11〜14 のテスト)と `demo/windows/run-demo.cmd`: `cfg(windows)` で、この macOS では実行もコンパイルもされない。
- ubuntu と windows でのビルドとテスト、最小の Rust(1.100.0)での `msrv-test`(このマシンの rustc は 1.100.0 で、MSRV と同じなので、ここでの結果が、MSRV での確認を兼ねる)、起動の性能(`startup-bench`)、`cargo audit`: CI の仕事。CI Pipeline(この計画の後の段階)が、この確認を受け持つ。
- 権限(`0o555`)とシンボリックリンクのテスト(U4)は、実行時に効果を確かめ、効かない環境(root、Windows)では、確かめずに通る。このマシンでは、実際に確かめた(通った)。

## 判定の材料(Target Verification Matrix は `build-and-test-summary.md`)
