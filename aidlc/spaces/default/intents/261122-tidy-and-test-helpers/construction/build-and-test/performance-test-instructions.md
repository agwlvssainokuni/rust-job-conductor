# 性能のテストの手順(Performance Test Instructions)

この intent に性能の目標は無い。既存の性能のテストが変わらず通ることだけを確かめる。

```bash
cargo test --locked -p conductor-core --test plan_perf --test flow_perf --test env_merge_perf
cargo test --locked -p conductor-runtime --test nfr_performance
```

- `median` の補助を core の `tests/common` に寄せた(T30〜T35)。上限と測り方は変えていない。結果: 通った。
- decide の規則 5(全ジョブの走査)を消したので、実行してよいかの判断は少し軽くなるはずだが、測る目標は立てていない。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
