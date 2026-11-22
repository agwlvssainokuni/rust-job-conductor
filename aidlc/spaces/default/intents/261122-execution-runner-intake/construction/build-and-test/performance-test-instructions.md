# 性能のテストの手順(Performance Test Instructions)

この intent に性能の目標は無い。既存の性能のテストが変わらず通ることだけを確かめる。

```bash
cargo test --locked -p conductor-core --test plan_perf --test flow_perf --test env_merge_perf
```

- `plan_perf.rs`、`flow_perf.rs` は `JobExecution` の組み立てだけ直した(T4)。上限と測り方は変えていない。結果: 通った。
- 段 2 でリトライのたびのコマンドの分割がなくなるので、リトライの多い場合は少し速くなるはずだが、測る目標は立てていない。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
