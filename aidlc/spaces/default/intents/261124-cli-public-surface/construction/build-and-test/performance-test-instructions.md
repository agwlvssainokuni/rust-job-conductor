# 性能のテストの手順(Performance Test Instructions)

この intent では、本番の処理を変えていない(公開範囲を狭め、テストを移しただけ)。性能のテストは足していない。

```bash
cargo test --locked -p conductor-cli
```

- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
