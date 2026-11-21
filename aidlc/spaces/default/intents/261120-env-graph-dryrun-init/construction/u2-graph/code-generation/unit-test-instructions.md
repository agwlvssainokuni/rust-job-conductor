# 単体テストの手順(Unit Test Instructions): U2 u2-graph(`conductor graph`)

## 実行環境

- 既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定ファイルと、新しい依存は、足さない。
- 実バイナリのテストは、既存の補助の実行ファイル `conductor-cli-test-helper` と、`conductor` の実バイナリを使う。
- 最小の Rust(1.100)でも通ること(CI の `msrv-test`)。ローカルでは、主な開発環境(macOS)で実行する。ubuntu と windows は CI で確認する。

## この Unit の範囲を絞った実行のコマンド

どのコマンドも、この Unit が足す、または直すテストのファイルだけを対象にする(プロジェクト全体は実行しない)。

```bash
# core: 型、辺、重ね合わせ、「対応する形」の判定、レイアウトの正解データ、性能
cargo test --locked -p conductor-core --test graph_types --test graph_edges --test graph_overlay --test graph_drawable --test graph_layout_oracle --test graph_perf

# core: 既存の単体テストと、既存の流れのテスト(遷移の解決の切り出しが、結果を変えていないことを確かめる)
cargo test --locked -p conductor-core --lib --test flow_sequences --test retry_flow

# cli: 整形、流れ(偽の Runtime)、実バイナリ、README
cargo test --locked -p conductor-cli --test graph_report --test flow_graph --test binary_graph --test docs_graph

# cli: メッセージの網羅、文字のエスケープ、引数の文法(新しいコマンドとメッセージの確認)
cargo test --locked -p conductor-cli --test report_messages --test escape_table --test args_grammar --test flow_aux
```

既存のテスト(流れ、状態、デモなど)が壊れていないことは、`cargo test --locked --workspace` で確かめる。これは、この Unit のコマンドではなく、検証のコマンド(`cargo fmt`、`clippy`、`cargo llvm-cov`)と、Build and Test が行う。

## テストの数と種類(Standard)

| 部品 | ファイル | 数 | 種類 |
|------|----------|----|------|
| 図の型 | `crates/core/tests/graph_types.rs` | 約 4 | 単体 |
| 辺の取り出し | `crates/core/tests/graph_edges.rs` | 8 | 単体 |
| 状態の重ね合わせ | `crates/core/tests/graph_overlay.rs` | 5 | 単体 |
| 「対応する形」の判定 | `crates/core/tests/graph_drawable.rs` | 6 | 単体 |
| レーンの配置 | `crates/core/tests/graph_layout_oracle.rs` | 約 4(537 個の例を含む) | 単体(正解データ) |
| 性能 | `crates/core/tests/graph_perf.rs` | 2 | 非機能 |
| 整形(text、Mermaid、辺の一覧) | `crates/cli/tests/graph_report.rs` | 8 | 単体 |
| 流れ | `crates/cli/tests/flow_graph.rs` | 8 | 統合(偽の `Runtime`) |
| 実バイナリ | `crates/cli/tests/binary_graph.rs` | 4 | 統合(実バイナリ) |
| README | `crates/cli/tests/docs_graph.rs` | 2 | 単体 |

テスト名は、日本語で書く(`team.md`)。

## 期待するカバレッジ

- 行カバレッジ 80% 以上(プロジェクト全体)。`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(CI の `coverage` と同じ)。これは、プロジェクト全体のしきい値で、Build and Test が、全体のテストで確かめる。この Unit だけの範囲を絞った実行では、判定しない。しきい値は、下げない。
- この Unit が足す新しいコード(`graph/` の 4 つのモジュール、`cli` の `graph.rs` と整形)は、ほとんどの行が、上のテストで通ること。

## モックとスタブの方針

- 流れのテストは、既存の方針どおり、偽の `Runtime`、`Sink`、`Env`、`Signals` を注入する。偽の `Runtime` は、ステートの読み取り(`read_state_raw`)の結果を、テストが決めた値で返し、ステートの書き込みと削除が呼ばれたら、テストを失敗させる。
- レーンの配置のテストは、偽を使わず、`layout-oracle.md`(コピーした正解データ)を、そのまま読み込む。

## テストデータの管理

- レイアウトの正解データ(`crates/core/tests/fixtures/graph_layout_oracle.md`)は、`layout-oracle.md` と同じ内容で、変更しない(実際の git の出力から作った。537 個)。
- 秘密の値の検査には、識別しやすい固定の文字列(例: `s3cr3t-VALUE-do-not-print`)を使い、標準出力、標準エラーのどこにも無いことを、検査する。
- ファイルは、各テストの一時ディレクトリに作る(終わると消える)。
