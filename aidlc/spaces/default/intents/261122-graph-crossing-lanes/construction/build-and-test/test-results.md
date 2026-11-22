# テスト結果(Test Results): graph の text 表示の改善 その 3

実行した日: 2026-11-22。環境: macOS(Darwin)、`rustc 1.100.0`。リポジトリの状態: `main` の、コミット `0ca6c67`。

## ビルドと静的な確認

- `cargo build --workspace --locked`: 成功。`cargo fmt --all -- --check`: 差分なし。`cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告なし。

## テスト(プロジェクト全体)

- `cargo test --locked --workspace`: **1,075 件が成功、失敗 0 件**(以前からの `#[ignore]` が 1 件)。
- 範囲を絞ったコマンド(`unit-test-instructions.md` の 5 つ)は、コード生成の段階で成功済み(graph_layout_oracle 7、graph_drawable 12、graph_lane_polish 13、graph_branch_lanes 8、graph_crossing_lanes 9、graph_perf 3、graph_types 5、graph_overlay 5、graph_report 18、binary_graph 8、docs_graph 6)。

## カバレッジ(プロジェクト全体、1 回の全体の実行)

- コマンド: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`。終了コード 0。
- 行カバレッジ **97.90%**(9,424 行のうち未通過 198)。`core/src/graph/layout.rs` 96.58%、`cli/src/report/graph_text.rs` 99.43%。

## 依存と README

- `Cargo.toml`、`Cargo.lock`、`crates/*/Cargo.toml` の、`dfdc038` 以降の差分: なし。README の必須語句 7 つ: すべて残っている。

## 失敗の詳細

なし。

## 実行しなかったもの(理由つき)

- Windows と ubuntu のテスト、最小の Rust(1.100.0)での `msrv-test`(rustc が同じ版なので、ここでの結果が兼ねる)、`startup-bench`、`cargo audit`: CI の仕事。この変更は OS に依存するコードを足していない。push 後に `ci.yml` の結果で確かめる。
