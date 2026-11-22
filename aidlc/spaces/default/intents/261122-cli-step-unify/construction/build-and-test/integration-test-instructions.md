# 結合テストの手順(Integration Test Instructions): run と pipeline の重複をなくすリファクタリング

ユニットは 1 つで、外部のサービスはない。結合の確かめは、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する流れのテストと、実バイナリのテストで行う(既存のまま)。

```bash
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test flow_run_id --test signals_order
cargo test --locked -p conductor-cli --test binary_basic
```

- 既存のテストは 1 行も変えずに通る(`git diff 21fef24..HEAD -- crates/cli/tests` は追加だけ)。
- 新しい 3 本: `flow_run.rs` の、記録の変わらないスキップでは消さないこと。`flow_pipeline.rs` の、スキップで完了して消すこと、表示のあとで消し、削除の失敗は 65 であること。
