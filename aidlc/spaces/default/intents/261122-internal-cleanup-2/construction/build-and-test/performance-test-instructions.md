# 性能のテストの手順(Performance Test Instructions)

```bash
cargo test --locked -p conductor-core --test flow_perf --test env_merge_perf
```

- debug(CI と同じ)で、5 回の中央値を上限と比べる。入力はテストの中で組み立てる(ジョブ 1,000 個、`.env` 10,000 項目)。
- 上限: A2 50 ms、A3a 50 ms、A3b 100 ms(`construction/functional-design/functional-spec.md` §5)。
- 結果: A2 約 5.3 ms、A3a 約 5.1 ms、A3b 約 8.9 ms。上限まで約 10 倍の余裕がある。直す前の形(二乗の時間)なら 290〜1,700 ms で確実に落ちる。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は今回の変更の対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`(SM2)、`construction/code-generation/code-generation-plan.md`(Step 3、4)。
