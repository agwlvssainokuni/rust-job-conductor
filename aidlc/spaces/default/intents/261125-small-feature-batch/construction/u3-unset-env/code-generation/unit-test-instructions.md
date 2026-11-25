# 単体テストの手順: u3-unset-env

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、runtime は `crates/runtime/tests/`、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`、cli の内部のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

```bash
cargo test -p conductor-core --test env_definition --test env_file_parse --test env_merge --test env_merge_perf --test nfr_security --locked
```

```bash
cargo test -p conductor-runtime --test runner_basic --test env_loader --locked
```

```bash
cargo test -p conductor-cli --test flow_env --test flow_validate --test flow_graph --test flow_dry_run --test binary_env --test unix_demo --locked
```

- `plan_report` と `docs` の内部のテストは `cargo test -p conductor-cli --lib --locked -- tests::plan_report tests::docs` で走らせる。
- 型の変更はテストの組み立てに及ぶため、最後に `cargo test --workspace --locked` で全体が通ることも確かめる。
- `windows_demo.rs` と Windows の大文字小文字の消し方は、CI の windows で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。ユニットの確認のコマンドと Build and Test が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。

## 偽物と差し替え

- core のテストは差し替え無し。runtime のテストは今の補助バイナリで環境を読む。cli の流れのテストは偽の `Runtime` の記録で消す名前を確かめる。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- 定義と `.env` は各テストの中の文字列で作る。親の環境の変数は、子のプロセスにだけ渡す(テストのプロセスの環境を変えない)。
