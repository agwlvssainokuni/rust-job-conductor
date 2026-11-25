# 単体テストの手順: u7-watch-follow

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。
- core の統合テストは `crates/core/tests/`、cli の流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`・`Signals`)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`、cli の内部のテストは `crates/cli/src/tests/`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。

```bash
cargo test -p conductor-core --test pipeline_activity --test attempt_flow --locked
```

```bash
cargo test -p conductor-cli --test flow_aux --test flow_retry --test flow_run --test flow_pipeline --test binary_basic --test unix_demo --locked
```

```bash
cargo test -p conductor-cli --lib --locked -- tests::args_grammar tests::docs tests::report_messages
```

- `pipeline_activity` はこのユニットで新しく作るファイル。Step 1(変更の前)では、このファイルを除いて走らせる。
- 版の更新と U6 の流れの直しはテストの広い範囲に及ぶため、最後に `cargo test --workspace --locked` で全体が通ることも確かめる。
- `windows_demo.rs` と Windows の追記の表示は、CI の windows で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。ユニットの確認のコマンドと Build and Test が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。

## 偽物と差し替え

- core のテストは差し替え無し。cli の流れのテストは偽の `Runtime`(読み込みの並び、`now()`、`wait` の結果)と偽の `Signals` で動かし、本当には待たない。画面を消す文字列の期待は `cfg!(unix)` で分ける。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- ステートと定義は各テストの中の文字列か、今の補助で作る。
