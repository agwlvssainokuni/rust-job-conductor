# 単体テストの手順(Unit Test Instructions): U3 u3-dry-run(`--dry-run` と `--assume`)

## 実行環境

- 既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定ファイルと、新しい依存は、足さない。
- 実バイナリのテストは、既存の補助の実行ファイル `conductor-cli-test-helper` と、`conductor` の実バイナリを使う。
- 最小の Rust(1.100)でも通ること(CI の `msrv-test`)。ローカルでは、主な開発環境(macOS)で実行する。ubuntu と windows は CI で確認する。

## この Unit の範囲を絞った実行のコマンド

どのコマンドも、この Unit が足す、または直すテストのファイルだけを対象にする(プロジェクト全体は実行しない)。

```bash
# core: 計画の型、模擬、仮定、run の計画、性能
cargo test --locked -p conductor-core --test plan_types --test plan_pipeline --test plan_assume --test plan_run --test plan_perf

# core: 既存の単体テストと、既存の流れのテスト(core を変えていないことの確認)
cargo test --locked -p conductor-core --lib --test flow_sequences --test retry_flow

# cli: 整形、流れ(偽の Runtime)、実バイナリ、README
cargo test --locked -p conductor-cli --test plan_report --test flow_dry_run --test binary_dry_run --test docs_dry_run

# cli: 引数の文法、メッセージの網羅、文字のエスケープ、既存の流れ(prepare の切り出しが、本番の動きを変えていないことの確認)
cargo test --locked -p conductor-cli --test args_grammar --test report_messages --test escape_table --test flow_pipeline --test flow_run --test flow_retry --test flow_env
```

既存のテスト(状態、シグナル、デモなど)が壊れていないことは、`cargo test --locked --workspace` で確かめる。これは、この Unit のコマンドではなく、検証のコマンド(`cargo fmt`、`clippy`、`cargo llvm-cov`)と、Build and Test が行う。

## テストの数と種類(Standard)

| 部品 | ファイル | 数 | 種類 |
|------|----------|----|------|
| 計画の型 | `crates/core/tests/plan_types.rs` | 約 3 | 単体 |
| 模擬と計画(`pipeline`) | `crates/core/tests/plan_pipeline.rs` | 8 | 単体 |
| 仮定 | `crates/core/tests/plan_assume.rs` | 5 | 単体 |
| `run` の計画 | `crates/core/tests/plan_run.rs` | 5 | 単体 |
| 性能 | `crates/core/tests/plan_perf.rs` | 1 | 非機能 |
| 整形 | `crates/cli/tests/plan_report.rs` | 7 | 単体 |
| 流れ(実行との一致を含む) | `crates/cli/tests/flow_dry_run.rs` | 9 | 統合(偽の `Runtime`) |
| 実バイナリ | `crates/cli/tests/binary_dry_run.rs` | 4 | 統合(実バイナリ) |
| README | `crates/cli/tests/docs_dry_run.rs` | 2 | 単体 |

テスト名は、日本語で書く(`team.md`)。

## 期待するカバレッジ

- 行カバレッジ 80% 以上(プロジェクト全体)。`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(CI の `coverage` と同じ)。これは、プロジェクト全体のしきい値で、Build and Test が、全体のテストで確かめる。この Unit だけの範囲を絞った実行では、判定しない。しきい値は、下げない。
- この Unit が足す新しいコード(`plan/`、`cli` の `dry_run.rs` と整形)は、ほとんどの行が、上のテストで通ること。

## モックとスタブの方針

- 流れのテストは、既存の方針どおり、偽の `Runtime`、`Sink`、`Env`、`Signals` を注入する。偽の `Runtime` は、ステートの書き込みと削除、`run_job`、シグナルの準備が呼ばれたら、テストを失敗させる(`--dry-run` のとき)。実行との一致のテストでは、偽の `Runtime` が、仮定どおりの終了コードを返す。
- 計画の生成(`core`)のテストは、偽を使わず、定義とステートの内容を、直接組み立てる。

## テストデータの管理

- 秘密の値の検査には、識別しやすい固定の文字列(例: `s3cr3t-VALUE-do-not-print`)を使い、標準出力、標準エラーのどこにも無いことを、検査する。`--assume` の誤りの検査には、ジョブ ID の固定の文字列(例: `job-secret-id`)を使い、誤りの表示に含まれないことを検査する。
- ファイルは、各テストの一時ディレクトリに作る(終わると消える)。
