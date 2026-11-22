# テストの手順(Unit Test Instructions): 動きを変えない整理のリファクタリング

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(段の前と後で件数を比べる。基準: 1,123 通過、1 無視)
cargo test --locked --workspace

# 段 1(cli の流れ)
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_graph --test flow_aux --test flow_run_id

# 段 2(型、ステート、表示)
cargo test --locked -p conductor-cli --test report_messages
cargo test --locked -p conductor-runtime --test state_read --test state_read_raw --test state_write --test flow_with_state

# 段 3(定義の読み取り、模擬)
cargo test --locked -p conductor-core

# 段 4(graph のテスト、converter、README)
cargo test --locked -p conductor-core --test graph_shape_skip --test graph_shape_same_target --test graph_shape_both --test graph_shape_branch --test graph_drawable
cargo test --locked -p conductor-converter
cargo test --locked -p conductor-cli --test docs_graph
```

(ファイル名が違うときは、実際のテストのファイル名に合わせる。)

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| SM1〜SM3(段 1〜3) | 既存のテスト(無変更) | 件数が基準と同じ。`git diff` で段 1〜3 のコミットがテストのファイルを変えていない(例外: `report_messages.rs` の組み立ての値) |
| SM4(段 4) | 組み直した graph のテスト、改名した converter のテスト、README の検査 | まとめた 6 件の一覧を記録。ほかの入力と期待を落としていない |
| SM5(動きを変えない) | 全体 | 出力の期待の文字列に差分がない |
| SM6(合格の条件) | fmt、clippy、全テスト、カバレッジ 80% | 全体 |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。全体のしきい値は、Build and Test が確かめる。段 1b で通らない枝が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う。段 4c で、偽物の `wait` を共通の関数にする(フィールドは残す)。

## テストデータ

- 既存のテストの定義をそのまま使う。
