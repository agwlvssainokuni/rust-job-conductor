# 性能のテストの手順(Performance Test Instructions): run と pipeline の重複をなくすリファクタリング

この変更は、同じ処理を関数にまとめただけで、処理の量も順序も変わらない。新しい性能のテストは要らない。

- 起動の性能(`nfr_perf.rs`、CI の `startup-bench`)は、既存のとおり(警告のみ)。

```bash
cargo test --locked -p conductor-cli --test nfr_perf
```
