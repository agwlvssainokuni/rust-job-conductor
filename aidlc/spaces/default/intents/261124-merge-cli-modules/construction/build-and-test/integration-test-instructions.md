# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。cli の流れのテスト(偽の `Runtime`・`Sink`・`Env`・`Signals`)と実バイナリのテストを使う。

```bash
# cli: 単体 172 件、統合 284 件(リトライ、graph、init、表示のエスケープ、シグナルを含む)
cargo test --locked -p conductor-cli
```

- 処理のコードは変えていないので、`run`・`pipeline`(リトライを含む)、`graph`、`init`、表示のエスケープ、シグナルの監視の流れのテストは、期待を変えずに通る。
- シグナルの監視の入口(`platform`)の Windows での動きは、push の後の CI の `test`(windows)で確かめる。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,149 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜4、`construction/code-generation/unit-test-instructions.md`。
