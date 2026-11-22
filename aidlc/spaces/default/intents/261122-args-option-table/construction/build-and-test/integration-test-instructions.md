# 統合テストの手順(Integration Test Instructions)

`parse_args` は純粋な関数で、cli の統合テスト `args_grammar.rs` が直接呼んで確かめる。実バイナリを動かすテスト(`binary_basic.rs` の convert など)と流れのテストも、変えずに通る。

```bash
cargo test --locked -p conductor-cli --test args_grammar
cargo test --locked -p conductor-cli --test binary_basic
cargo test --locked --workspace
```

結果は `test-results.md`。
