# 性能のテストの手順(Performance Test Instructions)

この intent では、型から読まれない項目を消しただけで、計算の手順は変えていない。性能のテストは足していない。

```bash
cargo test --locked -p conductor-core
```

- 既存の性能のテスト(`graph_lookup_perf.rs`、`env_merge_perf.rs`、`env_parse_perf.rs` など)は、確かめの行を書き直したうえで、上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
