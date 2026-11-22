# テストの手順(Unit Test Instructions): 動きを変えない整理の第 2 弾

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,128 通過、1 無視、0 失敗)
cargo test --locked --workspace

# 段 2 の性能のテスト(debug。CI と同じ)
cargo test --locked -p conductor-core --test flow_perf --test env_merge_perf

# 段 3 の流れ
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_dry_run --test flow_run_id --test flow_env

# 段 4
cargo test --locked -p conductor-cli --test args_grammar --test report_messages
cargo test --locked -p conductor-runtime --test flow_with_state
cargo test --locked -p conductor-core
```

(ファイル名が違うときは、実際のテストのファイル名に合わせる。)

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| SM1(段 1) | 既存のテスト(T8 はコメントだけ) | 全体が通り、件数が基準と同じ |
| SM2(段 2) | `flow_perf.rs`(A2)、`env_merge_perf.rs`(A3a、A3b)、既存のテスト(T1) | 性能のテストが上限(50/50/100 ms、5 回の中央値)の中。直す前と後の時間を記録 |
| SM3(段 3) | 既存の流れのテスト(無変更) | 誤りの順序と表示がそのまま通る |
| SM4(段 4) | §6 の単体テスト 4 件、T2〜T7、T9 | 型の変更のあとも期待の文字列が同じ。`flow_with_state.rs` は 20 件 |
| SM5(動きを変えない) | 全体 | 出力の期待の文字列に差分がない。テストのファイルの差分は T1〜T9 と新しいファイルだけ |
| SM6(合格の条件) | 全体 | fmt、clippy、全テスト、カバレッジ 80% |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。重複がまとまり、到達しない分岐が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う。

## テストデータ

- 性能のテストは、テストの中で入力を組み立てる(ジョブ 1,000 個、`.env` 10,000 項目)。ファイルを足さない。
