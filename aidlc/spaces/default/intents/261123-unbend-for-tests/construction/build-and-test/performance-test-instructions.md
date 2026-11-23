# 性能のテストの手順(Performance Test Instructions)

この intent では性能のテストを足していない。`flow_edges` を `pub(crate)` にした 1j で、既存の性能のテスト P4 の前後の時間を記録した。

```bash
cargo test --locked -p conductor-core
```

- 結果(手元、debug): P4 は変える前 117.2 ms、後 121.3〜132.3 ms(上限 600 ms)。差は揺れの範囲。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。CI では 3 つの OS で通った。

上流: `construction/code-generation/code-generation-plan.md`(Step 6)、`construction/code-generation/unit-test-instructions.md`。
