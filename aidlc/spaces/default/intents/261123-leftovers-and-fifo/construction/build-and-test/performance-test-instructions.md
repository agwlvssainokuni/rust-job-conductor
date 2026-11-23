# 性能のテストの手順(Performance Test Instructions)

```bash
cargo test --locked -p conductor-core --test definition_perf --test env_refs_perf
```

- debug(CI と同じ)で、5 回の中央値を上限と比べる。入力はテストの中で組み立てる。
- 上限: P1 350 ms、P2 1,600 ms、P3 40 ms(`construction/functional-design/functional-spec.md` §5.2。D9 の決定で、直した後の見込みの約 5 倍)。
- 結果(手元): P1 41 ms、P2 約 240 ms、P3 約 7〜10 ms。直す前は約 630・2,030・620 ms で、直す前の木では上限で落ちた。
- P3 の余裕は約 4 倍と一番狭い。CI(特に Windows と計測つきの coverage)で揺れたら、上限を 80 ms 程度に上げる。
- `-v` の図と `record_skip` は、性能のテストを足さず、前後の時間だけ記録した(D8)。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`(Step 7)。
