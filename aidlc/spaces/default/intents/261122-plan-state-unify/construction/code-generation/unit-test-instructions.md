# テストの手順(Unit Test Instructions): `--dry-run` の模擬の重複をなくすリファクタリング

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲)

```bash
# 全体(変更の前と後で、件数を比べる)
cargo test --locked --workspace

# 更新の関数の単体のテスト(core へ移った record.rs の中)と、模擬
cargo test --locked -p conductor-core --lib flow::record
cargo test --locked -p conductor-core --test plan_pipeline --test plan_run --test plan_perf --test flow_sequences

# 本番の流れ(ステートの書き込み)
cargo test --locked -p conductor-runtime --test flow_with_state
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_dry_run

# 比べる確認(一時のテスト。Step 1 で作り、Step 5 で削除する)
SEED=20261122 cargo test --locked -p conductor-runtime --test tmp_plan_compare -- --ignored --nocapture
SEED=7 cargo test --locked -p conductor-runtime --test tmp_plan_compare -- --ignored --nocapture
SEED=123456789 cargo test --locked -p conductor-runtime --test tmp_plan_compare -- --ignored --nocapture
```

(`flow_dry_run` など、名前が違うときは、実際のテストのファイル名に合わせる。)

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(更新が 1 か所) | ビルドと clippy が通る。`record.rs` の単体のテスト(中身は無変更) | `crates/core/src/flow/record.rs` |
| SM2(模擬もスキップを記録) | 比べる確認(違い 0 件)、手書きのステートの形 | 一時のテスト、`plan_pipeline.rs` |
| SM3(動きが変わらない、速さ) | 既存のテストが無変更で通る。性能(1 秒以内) | 全体、`plan_perf.rs` |
| SM4(仕様書) | 読み合わせ | `docs/spec/rust` |
| SM5(合格の条件) | fmt、clippy、全テスト、カバレッジ 80% | 全体 |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。全体のしきい値は、Build and Test が確かめる。

## モックとスタブ

- 更新と模擬は純粋な関数なので、偽物は要らない。cli の流れのテストは、既存の偽物を使う(変えない)。

## テストデータ

- 既存のテストの定義をそのまま使う。新しい 2 本の定義とステートは、テストの中で組み立てる。
