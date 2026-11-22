# 統合テストの手順(Integration Test Instructions)

この intent は動きを変えない整理なので、新しい統合テストは足していない。既存の統合テスト(各クレートの `tests/`、実バイナリを動かすテスト、デモのテスト)がそのまま通ることで、利用者から見える動きが変わっていないことを確かめる。

```bash
cargo test --locked --workspace
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_graph
cargo test --locked -p conductor-cli --test report_messages
```

結果は `test-results.md`。
