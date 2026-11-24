# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。cli の統合テスト(`tests/`)は、公開に残した入口(`run_cli`、`run_cli_with` と偽の `Runtime`・`Sink`・`Env`・`Signals`)と実バイナリ(`conductor-cli-test-helper`)だけで確かめる 294 件が残る。狭めた項目を直接呼ぶ 162 件は `src/tests/` の単体テストへ移った。

```bash
# cli: 単体(src/tests/ を含む 164 件)と統合(294 件)
cargo test --locked -p conductor-cli
```

- `flow_init.rs` は、期待のひな形を原本の写しで作る形に直し、期待のバイト列は同じまま通る。
- `nfr_source.rs` の 6 件は、D2 の範囲で通る。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,152 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜8、`construction/code-generation/unit-test-instructions.md`。
