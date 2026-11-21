# 単体テストの手順(Unit Test Instructions): U1 u1-env-injection(環境変数の注入と `.env`)

## 実行環境

- 既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定ファイルと、新しい依存は、足さない。
- 実バイナリのテストは、既存の補助の実行ファイル `conductor-cli-test-helper`(テスト専用。配布しない)の、`env <NAME>...` モードを使う。
- 最小の Rust(1.100)でも通ること(CI の `msrv-test`)。ローカルでは、主な開発環境(macOS)で実行する。ubuntu と windows は CI で確認する。

## この Unit の範囲を絞った実行のコマンド

どのコマンドも、この Unit が足す、または直すテストのファイルだけを対象にする(プロジェクト全体は実行しない)。

```bash
# core: 型、定義の検査、.env の解析、層の統合(単体)
cargo test --locked -p conductor-core --test env_types --test env_definition --test env_file_parse --test env_merge

# core: 既存の単体テスト(型の作り方の修正と、誤りの種類の追加を確かめる)
cargo test --locked -p conductor-core --lib

# runtime: .env の読み取りと、起動の口の環境
cargo test --locked -p conductor-runtime --test env_loader --test runner_env

# cli: 流れ(偽の Runtime)、実バイナリ、README
cargo test --locked -p conductor-cli --test flow_env --test binary_env --test docs_env

# cli: メッセージの網羅と、文字のエスケープ(新しいメッセージの種類の確認)
cargo test --locked -p conductor-cli --test report_messages --test escape_table
```

既存のテスト(流れ、状態、デモなど)が壊れていないことは、`cargo test --locked --workspace` で確かめる。これは、この Unit のコマンドではなく、検証のコマンド(`cargo fmt`、`clippy`、`cargo llvm-cov`)と、Build and Test が行う。

## テストの数と種類(Standard)

| 部品 | ファイル | 数 | 種類 |
|------|----------|----|------|
| 型と誤りの種類 | `crates/core/tests/env_types.rs` | 約 4 | 単体 |
| 定義の検査 | `crates/core/tests/env_definition.rs` | 8 | 単体 |
| `.env` の解析 | `crates/core/tests/env_file_parse.rs` | 8 | 単体 |
| 層の統合 | `crates/core/tests/env_merge.rs` | 8 | 単体 |
| `.env` の読み取り | `crates/runtime/tests/env_loader.rs` | 7 | 統合(一時ディレクトリ) |
| 起動の口 | `crates/runtime/tests/runner_env.rs` | 5 | 統合(実プロセス) |
| 流れ | `crates/cli/tests/flow_env.rs` | 8 | 統合(偽の `Runtime`) |
| 実バイナリ | `crates/cli/tests/binary_env.rs` | 5 | 統合(実バイナリ) |
| README | `crates/cli/tests/docs_env.rs` | 2 | 単体 |

テスト名は、日本語で書く(`team.md`)。

## 期待するカバレッジ

- 行カバレッジ 80% 以上(プロジェクト全体)。`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(CI の `coverage` と同じ)。これは、プロジェクト全体のしきい値で、Build and Test が、全体のテストで確かめる。この Unit だけの範囲を絞った実行では、判定しない。しきい値は、下げない。
- この Unit が足す新しいコード(`envfile` の 3 つのモジュール、`runtime` の読み取り、`cli` の新しい分岐)は、ほとんどの行が、上のテストで通ること。

## モックとスタブの方針

- 流れのテストは、既存の方針どおり、偽の `Runtime`、`Sink`、`Env`、`Signals` を注入する。偽の `Runtime` は、`load_env_files` の結果を、テストが決めた値で返し、`run_job` に渡された環境(並び)を記録する。
- `.env` の読み取りのテストは、偽を使わず、一時ディレクトリの実ファイルを使う。
- 環境変数を、プロセス全体に設定しない(並列のテストが干渉するため)。子のプロセスの環境の確認は、`conductor-cli-test-helper env` を、実バイナリ経由で起動する。

## テストデータの管理

- 秘密の値の検査には、識別しやすい固定の文字列(例: `s3cr3t-VALUE-do-not-print`)を使い、標準出力、標準エラー、ステートファイル、`Debug` の出力のどこにも無いことを、検査する。
- ファイルは、各テストの一時ディレクトリに作る(終わると消える)。大きさの上限のテストは、1 MiB のファイルを、その場で作る。
