# 単体テストの手順: u6-attempt-times

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、runtime は `crates/runtime/tests/`、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`、cli の内部のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

```bash
cargo test -p conductor-core --test attempt_flow --test time_format --test flow_sequences --test retry_flow --locked
```

```bash
cargo test -p conductor-runtime --test state_read --test state_write --test nfr_content --test flow_with_state --test run_id_state --locked
```

```bash
cargo test -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test flow_aux --test flow_graph --test flow_dry_run --test binary_basic --test unix_demo --locked
```

- `attempt_flow` はこのユニットで新しく作るファイル。Step 1(変更の前)では、このファイルを除いて走らせる。
- `docs` の内部のテストは `cargo test -p conductor-cli --lib --locked -- tests::docs` で走らせる。
- 型の変更と固定のステートの直しはテストの広い範囲に及ぶため、最後に `cargo test --workspace --locked` で全体が通ることも確かめる。
- `windows_demo.rs` は、CI の windows で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。ユニットの確認のコマンドと Build and Test が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。

## 偽物と差し替え

- core のテストは差し替え無し(時刻は `Timestamp::from_unix_seconds` で作る)。runtime のテストは一時ディレクトリにステートを書いて読む。cli の流れのテストは偽の `Runtime` の `now()` で時刻を固定し、偽の書き込みの記録で順と回数を確かめる。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- ステートと定義は各テストの中の文字列で作る。固定のステートに試行を足す直しは、確かめる内容を変えない。
