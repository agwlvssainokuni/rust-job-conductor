# 性能のテストの手順(Performance Test Instructions): `--dry-run` の模擬の重複をなくすリファクタリング

```bash
cargo test --locked -p conductor-core --test plan_perf
```

- 既存: 1,000 個のジョブがすべて実行する計画が 1 秒以内。
- 新規: 終点の後に 999 個のジョブをスキップする計画が 1 秒以内(約 0.1 秒。変更の前の更新の関数では、同じ形で 1.13 秒だった)。`apply_skip` を O(n) にしたので、本番の `pipeline` の同じ形も速くなる。
