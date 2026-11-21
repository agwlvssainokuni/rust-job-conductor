# 単体テストの手順(Unit Test Instructions): U4 u4-init(`conductor init`)

## 実行環境

- 既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定ファイルと、新しい依存は、足さない。
- 実バイナリのテストは、`conductor` の実バイナリを使う。
- 最小の Rust(1.100)でも通ること(CI の `msrv-test`)。ローカルでは、主な開発環境(macOS)で実行する。ubuntu と windows は CI で確認する。

## この Unit の範囲を絞った実行のコマンド

どのコマンドも、この Unit が足す、または直すテストのファイルだけを対象にする(プロジェクト全体は実行しない)。

```bash
# runtime: 排他的なファイルの作成
cargo test --locked -p conductor-runtime --test file_create

# cli: 型、ひな形(validate に通ること)、流れ(偽の Runtime)、実バイナリ、README
cargo test --locked -p conductor-cli --test init_types --test init_template --test flow_init --test binary_init --test docs_init

# cli: 引数の文法、メッセージの網羅、文字のエスケープ(新しいコマンドとメッセージの確認)
cargo test --locked -p conductor-cli --test args_grammar --test report_messages --test escape_table
```

既存のテスト(流れ、状態、デモなど)が壊れていないことは、`cargo test --locked --workspace` で確かめる。これは、この Unit のコマンドではなく、検証のコマンド(`cargo fmt`、`clippy`、`cargo llvm-cov`)と、Build and Test が行う。

## テストの数と種類(Standard)

| 部品 | ファイル | 数 | 種類 |
|------|----------|----|------|
| 型 | `crates/cli/tests/init_types.rs` | 約 3 | 単体 |
| ファイルの作成 | `crates/runtime/tests/file_create.rs` | 8 | 統合(一時ディレクトリ) |
| ひな形 | `crates/cli/tests/init_template.rs` | 7 | 単体(`validate` に通る確認を含む) |
| 流れ | `crates/cli/tests/flow_init.rs` | 9 | 統合(偽の `Runtime`) |
| 実バイナリ | `crates/cli/tests/binary_init.rs` | 5 | 統合(実バイナリ) |
| README | `crates/cli/tests/docs_init.rs` | 2 | 単体 |

テスト名は、日本語で書く(`team.md`)。

## 期待するカバレッジ

- 行カバレッジ 80% 以上(プロジェクト全体)。`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(CI の `coverage` と同じ)。これは、プロジェクト全体のしきい値で、Build and Test が、全体のテストで確かめる。この Unit だけの範囲を絞った実行では、判定しない。しきい値は、下げない。
- この Unit が足す新しいコード(`runtime` の `file_create`、`cli` の `init.rs` とひな形)は、ほとんどの行が、上のテストで通ること。

## モックとスタブの方針

- 流れのテストは、既存の方針どおり、偽の `Runtime`、`Sink`、`Env`、`Signals` を注入する。偽の `Runtime` は、`create_file` の引数(パス、内容、`force`)を記録し、テストが決めた結果(`Created`、`Replaced`、各 `CreateError`)を返す。`init` が、定義、ステートの読み取り、シグナルの準備を呼んだら、テストを失敗させる。
- ファイルの作成のテストは、偽を使わず、一時ディレクトリの実ファイルを使う。権限のテストは、実行時に効果を確かめ、効かない環境(root など)では、確かめずに通す(`cfg` は、`platform` の外に置かない)。

## テストデータの管理

- ひな形の原本は、`functional-spec.md` の日本語と英語のひな形(テストのコードの中に、そのまま書く。実装の出力から作らない)。
- ファイルは、各テストの一時ディレクトリに作る(終わると消える)。
