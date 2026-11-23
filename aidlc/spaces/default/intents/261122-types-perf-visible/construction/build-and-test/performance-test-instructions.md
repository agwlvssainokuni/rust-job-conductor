# 性能のテストの手順(Performance Test Instructions)

```bash
cargo test --locked -p conductor-core --test plan_simulate_perf --test graph_lookup_perf --test env_parse_perf
```

- debug(CI と同じ)で、5 回の中央値を上限と比べる。入力はテストの中で組み立てる(ジョブ 1,000 個、`.env` 10,000 行)。
- 上限: P1・P2 150 ms、P3 350 ms、P4 600 ms、P5 100 ms(`construction/functional-design/functional-spec.md` §5.2。D6 の決定で、直した後の見積もりの約 5 倍)。
- 結果(手元の M2): P1 22 ms、P2 22 ms、P3 約 97 ms、P4 約 120 ms、P5 約 17 ms。直す前は 95・104・369・689・103 ms。
- 上限は大きな退行だけを止める。直す前の形に戻ったことは検知できない(改善は時間の記録で示す)。
- P3 の余裕は 3.6 倍と小さい。CI(特に Windows と計測つきの coverage)で揺れたら、遷移の無い定義では位置の表を作らない形にするか、上限を上げるかを決める。
- 既存の性能のテスト(`plan_perf.rs`、`flow_perf.rs`、`env_merge_perf.rs`)は、組み立ての変更だけで、上限と測り方は変えていない。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`(Step 8)。
