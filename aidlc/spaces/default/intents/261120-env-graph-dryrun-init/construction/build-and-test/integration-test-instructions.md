# 統合テストの手順(Integration Test Instructions): v0.9.0 の機能拡張

Standard の戦略に従い、境界をまたぐ動き(ユニットのあいだの連携と、実バイナリを使う動き)の確認の手順を、まとめる。ユニットごとのテストは、各ユニットの `unit-test-instructions.md`(`construction/<unit>/code-generation/`)にある。

## 実行環境

- 既存の Rust のテストの実行環境(`cargo test`)。実バイナリのテストは、`conductor` の実バイナリと、テスト専用の補助の実行ファイル(`conductor-cli-test-helper`)を使う。
- 流れの統合テストは、偽の `Runtime`、`Sink`、`Env`、`Signals` を注入する(ファイルとプロセスを使わない)。

## 境界と、確認するもの

| 境界 | 確認すること | ファイル |
|------|--------------|----------|
| U1 → U3(環境変数の名前の要約を、計画が使う) | `.env` と `env` を持つ定義の `--dry-run` が、名前だけを示し、値を出さない | `crates/cli/tests/binary_dry_run.rs`、`crates/cli/tests/plan_report.rs` |
| U1 → U2(`graph` が `validate` と同じ検査をする) | `.env` の誤りで `graph` も 64 で、図を出さない。値が出力に無い | `crates/cli/tests/flow_graph.rs`、`crates/cli/tests/binary_graph.rs` |
| U1 → U4(ひな形が U1 の新しいキーを含む) | ひな形が、U1 の検査(`[pipeline.env]` の名前の規則と予約名)を通る | `crates/cli/tests/init_template.rs` |
| U3 と U4 | `init` で作ったひな形に、`conductor pipeline my-pipeline --dry-run` が、計画を出す | `crates/cli/tests/binary_init.rs` |
| 本番の流れと計画(U3) | 同じ定義、同じステート、仮定どおりの終了コードで、本番の `pipeline` が起動するジョブと、計画の「実行する」ジョブが一致する | `crates/cli/tests/flow_dry_run.rs` |
| 実行と辺の取り出し(U2) | `classify_and_decide` の次のジョブと、辺の行き先が、すべての結果で一致する | `crates/core/tests/graph_edges.rs` |
| 環境変数と実プロセス(U1) | 足した値が、子のプロセスの環境に見える。引き継いだ環境が残る。値が標準出力、標準エラー、ステートに出ない | `crates/runtime/tests/runner_env.rs`、`crates/cli/tests/binary_env.rs` |
| ステートの読み取り(U2、U3) | `graph --state` は完了したステートも読む。`--dry-run` は本番と同じ読み方。どちらも書き込みと削除をしない | `crates/cli/tests/flow_graph.rs`、`crates/cli/tests/flow_dry_run.rs` |

## 実行のコマンド

```bash
# 実バイナリの統合テスト(U1〜U4)
cargo test --locked -p conductor-cli --test binary_env --test binary_graph --test binary_dry_run --test binary_init

# 流れの統合テスト(偽の Runtime)
cargo test --locked -p conductor-cli --test flow_env --test flow_graph --test flow_dry_run --test flow_init

# 境界をまたぐ確認の全体(既存のテストを含む)
cargo test --locked --workspace
```

## カバレッジの目標

- プロジェクト全体の行カバレッジ 80% 以上(`cargo llvm-cov ... --fail-under-lines 80`)。

## テストデータの管理

- 秘密の値の検査には、識別しやすい固定の文字列(例: `s3cr3t-VALUE-do-not-print`)を使い、標準出力、標準エラー、ステートファイル、`Debug` の出力のどこにも無いことを検査する。
- ファイルは、各テストの一時ディレクトリに作る(終わると消える)。
