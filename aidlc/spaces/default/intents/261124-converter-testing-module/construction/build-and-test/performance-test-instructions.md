# 性能のテストの手順(Performance Test Instructions)

この intent では、変換の処理を変えていない(テストを移し、テストのための公開と使われない関数を消しただけ)。性能のテストは足していない。

```bash
cargo test --locked -p conductor-converter
```

- 展開の上限やエイリアスの爆発を確かめるテスト(時間の上限つきの実行 `with_timeout` を使うもの)は、`src/tests/read_resolve.rs` へ移り、30 秒の上限の中で通った。
- 起動の性能(`nfr_perf.rs`、`startup-bench`)は対象外で、従来どおり警告のみ。

上流: `construction/code-generation/unit-test-instructions.md`、`construction/code-generation/code-generation-plan.md`。
