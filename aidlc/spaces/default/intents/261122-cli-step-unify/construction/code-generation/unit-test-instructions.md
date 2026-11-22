# テストの手順(Unit Test Instructions): run と pipeline の重複をなくすリファクタリング

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ)

```bash
# cli の全テスト(変更の前と後で、件数が同じで、すべて通ること)
cargo test --locked -p conductor-cli

# 特に見る流れのテスト(書き込みの順序と終了コード)
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test flow_run_id --test signals_order

# 通っていない枝の確認(行の表示で step.rs、run.rs、pipeline.rs を見る)
cargo llvm-cov --locked -p conductor-cli --text --ignore-filename-regex 'bin/conductor-(cli-)?test-helper'
```

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(手順が 1 つの関数にまとまる) | ビルドと clippy が通り、run と pipeline が共通の手順を呼ぶ | `crates/cli/src/step.rs` |
| SM2(動きは変わらない) | 既存のテストが変えずにすべて通る。通っていない枝の流れのテストを足す | `flow_run.rs`、`flow_pipeline.rs` ほか |
| SM3(仕様書) | 05 と 02 の記述が実装に合う | (読み合わせ) |
| SM4(合格の条件) | fmt、clippy、全テスト、カバレッジ 80% | 全体 |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。全体のしきい値は、Build and Test が全体のテストで確かめる。

## モックとスタブ

- 流れのテストは、既存の偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する形(`crates/cli/tests/common/mod.rs`)を使う。新しい偽物は作らない。

## テストデータ

- 既存のテストの定義をそのまま使う。新しいテストの定義は、テストの中で組み立てる。
