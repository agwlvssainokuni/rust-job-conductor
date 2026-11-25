# 性能のテストの手順(Performance Test Instructions)

この intent では、処理を変えていない(モジュールの置き場所と可視性と `use` だけ)。性能のテストは足していない。

```bash
cargo test --locked --workspace
```

- 既存の性能のテストは、全テストの中で上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。`platform` の中継が 1 段減っても、呼ばれるのは起動時の 1 回だけで、差は測れないほど小さい。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
