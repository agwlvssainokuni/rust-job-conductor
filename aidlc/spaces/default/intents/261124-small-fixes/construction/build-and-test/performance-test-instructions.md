# 性能のテストの手順(Performance Test Instructions)

```bash
cargo test --locked -p conductor-core --test plan_simulate_perf
```

- 消した `終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる`(1 回の計測、上限 1 秒)と同じ場面を、P2 の `p2_終点のあとの_999_個のスキップの計画は_150ms_以内`(5 回の中央値、上限 150 ms)が確かめ続ける。
- `千個のジョブがすべて実行する計画は_1秒以内に作れる` は、名前と本体を変えずに `plan_simulate_perf.rs` へ移した。`plan_perf.rs` は無くなった。
- 3 件とも上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`(Step 5)。
