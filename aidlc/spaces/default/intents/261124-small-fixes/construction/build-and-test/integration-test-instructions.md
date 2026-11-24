# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテストと実バイナリのテストを使う。

```bash
# core: 計画の模擬の性能のテスト(移した 1 件を含む 3 件)
cargo test --locked -p conductor-core --test plan_simulate_perf

# cli: 引数の文法(init の既定)、ソースの検査、シグナル
cargo test --locked -p conductor-cli
```

- `signals_order.rs` と `signals_real.rs` は変えていない(Q2: B)。
- 消した cli の 2 件の既定の値は、`args_grammar` の init の既定のテストが確かめ続ける。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,149 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 5〜7、`construction/code-generation/unit-test-instructions.md`。
