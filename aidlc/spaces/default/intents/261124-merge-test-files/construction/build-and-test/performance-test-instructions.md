# 性能のテストの手順(Performance Test Instructions)

この intent では、性能のテストを移していない(core の `*_perf.rs` と cli の `nfr_perf.rs` は範囲の外)。本番のコードも変えていない。

```bash
cargo test --locked --workspace
```

- 既存の性能のテストは、全テストの中で上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。CI が名前で指す `nfr_perf` は変えていない。
- 統合テストの実行ファイルは、cli で 5 つ、converter で 1 つ、runtime で 2 つ減った。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
