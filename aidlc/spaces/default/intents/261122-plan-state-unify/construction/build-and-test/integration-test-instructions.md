# 結合テストの手順(Integration Test Instructions): `--dry-run` の模擬の重複をなくすリファクタリング

ユニットは 1 つで、外部のサービスはない。結合の確かめは、本番の流れのテスト(ステートの書き込み)と、`--dry-run` の流れのテスト(既存のまま)で行う。

```bash
cargo test --locked -p conductor-runtime --test flow_with_state
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline
cargo test --locked -p conductor-core --test plan_pipeline --test plan_run --test flow_sequences
```

- 既存のテストは変えずに通る(移した `record.rs` の単体のテストの、`use` の 1 行を除く)。
- 実装の後に、書き換える前の模擬(写し)と書き換えた `plan_pipeline` を、3 つの種、計 90 万例で比べ、違いは 0 件(開発担当の一時のテスト。確かめたあと削除)。
