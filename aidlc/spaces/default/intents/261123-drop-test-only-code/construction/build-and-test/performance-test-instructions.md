# 性能のテストの手順(Performance Test Instructions)

この intent では、実行の経路を変えていない(消したのはテストだけが使う実装、狭めたのは公開範囲、`use` の移動は同じ実体)。性能のテストは足していない。

```bash
cargo test --locked --workspace
```

- 既存の性能のテスト(core の `*_perf.rs`)は、全テストの中で上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。CI(`6908726`)では 3 つの OS で通った。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
